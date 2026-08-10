//! Backend-routed connection pool.

use async_trait::async_trait;
#[cfg(any(feature = "mysql", feature = "postgres"))]
use sqlx_core::sql_str::AssertSqlSafe;

use crate::{DalError, DalPool, ExecResult, Value, row::Row};

#[cfg(feature = "sqlite")]
use std::sync::{Arc, Mutex};

/// Bind each [`Value`] onto a sqlx query in order. Written as a macro because
/// the concrete `Query` type differs per backend and spelling out the generic
/// bounds costs more than it saves.
#[cfg(any(feature = "mysql", feature = "postgres"))]
macro_rules! bind_params {
    ($q:expr, $params:expr) => {{
        let mut q = $q;
        for p in $params {
            q = match p {
                Value::Null => q.bind(Option::<i64>::None),
                Value::Bool(b) => q.bind(*b),
                Value::Integer(i) => q.bind(*i),
                Value::Real(f) => q.bind(*f),
                Value::Text(s) => q.bind(s.as_str()),
                Value::Blob(b) => q.bind(b.as_slice()),
            };
        }
        q
    }};
}

/// Database backend the pool is connected to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// SQLite (`sqlite:` URL scheme).
    Sqlite,
    /// MySQL / MariaDB (`mysql:` URL scheme).
    MySql,
    /// PostgreSQL (`postgres:` / `postgresql:` URL scheme).
    Postgres,
}

impl Backend {
    /// Parse the backend from the scheme portion of a connection URL.
    pub fn from_url(url: &str) -> Result<Self, DalError> {
        let lower = url.to_ascii_lowercase();
        if lower.starts_with("sqlite:") {
            Ok(Backend::Sqlite)
        } else if lower.starts_with("mysql:") || lower.starts_with("mariadb:") {
            Ok(Backend::MySql)
        } else if lower.starts_with("postgres:") || lower.starts_with("postgresql:") {
            Ok(Backend::Postgres)
        } else {
            Err(DalError::UnsupportedUrl(url.to_string()))
        }
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Backend::Sqlite => "SQLite",
            Backend::MySql => "MySQL",
            Backend::Postgres => "Postgres",
        };
        f.write_str(name)
    }
}

/// Where a SQLite pool points.
#[cfg(feature = "sqlite")]
#[derive(Debug, Clone, PartialEq, Eq)]
enum SqliteTarget {
    /// A private in-memory database.
    Memory,
    /// A database file on disk.
    File(std::path::PathBuf),
}

/// deadpool manager that opens and hands out rusqlite connections.
#[cfg(feature = "sqlite")]
#[derive(Debug)]
struct SqliteManager {
    target: SqliteTarget,
}

/// A pooled connection. `rusqlite::Connection` is `Send` but not `Sync`, and
/// every call into it blocks, so it lives behind an `Arc<Mutex<_>>` that can be
/// cloned into `spawn_blocking`.
#[cfg(feature = "sqlite")]
type PooledConn = Arc<Mutex<rusqlite::Connection>>;

#[cfg(feature = "sqlite")]
impl deadpool::managed::Manager for SqliteManager {
    type Type = PooledConn;
    type Error = DalError;

    async fn create(&self) -> Result<PooledConn, DalError> {
        let target = self.target.clone();
        tokio::task::spawn_blocking(move || {
            let conn = match target {
                SqliteTarget::Memory => rusqlite::Connection::open_in_memory(),
                SqliteTarget::File(path) => rusqlite::Connection::open(path),
            }
            .map_err(map_rusqlite)?;
            // A pool means concurrent writers, and SQLite fails a busy write
            // immediately unless a timeout is set. Without this, adding the
            // pool would turn contention into spurious SQLITE_BUSY errors.
            conn.busy_timeout(std::time::Duration::from_secs(5))
                .map_err(map_rusqlite)?;
            Ok::<_, DalError>(Arc::new(Mutex::new(conn)))
        })
        .await
        .map_err(|e| DalError::Database(e.to_string()))?
    }

    async fn recycle(
        &self,
        _conn: &mut PooledConn,
        _metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<DalError> {
        // Deliberately a no-op. A local SQLite handle does not go stale the way
        // a network connection does, and a failed recycle would make deadpool
        // drop and reopen the connection — which for `:memory:` would silently
        // discard the entire database.
        Ok(())
    }
}

/// Connection pool for SQLite, backed by [`deadpool`].
#[cfg(feature = "sqlite")]
#[derive(Clone)]
pub struct SqlitePool {
    pool: deadpool::managed::Pool<SqliteManager>,
}

#[cfg(feature = "sqlite")]
impl std::fmt::Debug for SqlitePool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqlitePool")
            .field("status", &self.pool.status())
            .finish_non_exhaustive()
    }
}

/// Default pool size for file-backed databases. SQLite serialises writers, so
/// a large pool buys little; this leaves room for concurrent readers without
/// encouraging write contention.
#[cfg(feature = "sqlite")]
const DEFAULT_SQLITE_POOL_SIZE: usize = 8;

#[cfg(feature = "sqlite")]
impl SqlitePool {
    /// Open from a connection URL (`sqlite::memory:`, `sqlite:path.db`, `sqlite:///abs/path`).
    ///
    /// In-memory databases are pooled with a maximum size of one: each SQLite
    /// connection to `:memory:` gets its *own* private database, so a larger
    /// pool would hand out connections to different, empty databases.
    pub fn open(url: &str) -> Result<Self, DalError> {
        let target = parse_sqlite_target(url)?;
        let max_size = match target {
            SqliteTarget::Memory => 1,
            SqliteTarget::File(_) => DEFAULT_SQLITE_POOL_SIZE,
        };
        Self::build(target, max_size)
    }

    /// Open with an explicit maximum pool size.
    ///
    /// Ignored for in-memory databases, which are always capped at one
    /// connection for the reason given on [`SqlitePool::open`].
    pub fn open_with_max_size(url: &str, max_size: usize) -> Result<Self, DalError> {
        let target = parse_sqlite_target(url)?;
        let max_size = match target {
            SqliteTarget::Memory => 1,
            SqliteTarget::File(_) => max_size.max(1),
        };
        Self::build(target, max_size)
    }

    fn build(target: SqliteTarget, max_size: usize) -> Result<Self, DalError> {
        let pool = deadpool::managed::Pool::builder(SqliteManager { target })
            .max_size(max_size)
            .build()
            .map_err(|e| DalError::Database(e.to_string()))?;
        Ok(Self { pool })
    }

    /// Check out a connection and run a blocking rusqlite closure on it.
    ///
    /// The pooled object is held for the whole call, so the connection is not
    /// returned to the pool while the closure is still using it.
    async fn interact<F, R>(&self, f: F) -> Result<R, DalError>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<R, DalError> + Send + 'static,
        R: Send + 'static,
    {
        let obj = self
            .pool
            .get()
            .await
            .map_err(|e| DalError::Database(format!("sqlite pool: {e}")))?;
        let conn = PooledConn::clone(&obj);
        let result = tokio::task::spawn_blocking(move || {
            let guard = conn
                .lock()
                .map_err(|_| DalError::Database("sqlite mutex poisoned".into()))?;
            f(&guard)
        })
        .await
        .map_err(|e| DalError::Database(e.to_string()))?;
        drop(obj);
        result
    }
}

#[cfg(feature = "sqlite")]
fn parse_sqlite_target(url: &str) -> Result<SqliteTarget, DalError> {
    let rest = url
        .strip_prefix("sqlite:")
        .ok_or_else(|| DalError::UnsupportedUrl(url.to_string()))?;
    if rest == ":memory:" || rest.is_empty() {
        Ok(SqliteTarget::Memory)
    } else {
        Ok(SqliteTarget::File(
            rest.trim_start_matches("//").to_string().into(),
        ))
    }
}

#[cfg(feature = "sqlite")]
fn map_rusqlite(e: rusqlite::Error) -> DalError {
    DalError::Database(e.to_string())
}

/// A runtime-routed connection pool that dispatches to the appropriate
/// backend driver (rusqlite for SQLite, sqlx for MySQL/Postgres).
///
/// Construct with [`Pool::connect`]; the URL scheme determines the backend.
#[derive(Debug, Clone)]
pub enum Pool {
    /// SQLite-backed pool (rusqlite).
    #[cfg(feature = "sqlite")]
    Sqlite(SqlitePool),
    /// MySQL-backed pool.
    #[cfg(feature = "mysql")]
    MySql(sqlx_mysql::MySqlPool),
    /// PostgreSQL-backed pool.
    #[cfg(feature = "postgres")]
    Postgres(sqlx_postgres::PgPool),
}

impl Pool {
    /// Connect using the given URL. The backend is selected automatically
    /// based on the URL scheme (`sqlite:`, `mysql:`, `postgres:`).
    pub async fn connect(url: &str) -> Result<Self, DalError> {
        let backend = Backend::from_url(url)?;
        tracing::debug!(backend = ?backend, "ironroot-dal: connecting");
        match backend {
            Backend::Sqlite => Self::connect_sqlite(url).await,
            Backend::MySql => Self::connect_mysql(url).await,
            Backend::Postgres => Self::connect_postgres(url).await,
        }
    }

    #[cfg(feature = "sqlite")]
    async fn connect_sqlite(url: &str) -> Result<Self, DalError> {
        Ok(Pool::Sqlite(SqlitePool::open(url)?))
    }

    #[cfg(not(feature = "sqlite"))]
    async fn connect_sqlite(_url: &str) -> Result<Self, DalError> {
        Err(DalError::UnsupportedUrl(
            "sqlite support not compiled in (enable the `sqlite` feature)".into(),
        ))
    }

    #[cfg(feature = "mysql")]
    async fn connect_mysql(url: &str) -> Result<Self, DalError> {
        let pool = sqlx_mysql::MySqlPool::connect(url).await?;
        Ok(Pool::MySql(pool))
    }

    #[cfg(not(feature = "mysql"))]
    async fn connect_mysql(_url: &str) -> Result<Self, DalError> {
        Err(DalError::UnsupportedUrl(
            "mysql support not compiled in (enable the `mysql` feature)".into(),
        ))
    }

    #[cfg(feature = "postgres")]
    async fn connect_postgres(url: &str) -> Result<Self, DalError> {
        let pool = sqlx_postgres::PgPool::connect(url).await?;
        Ok(Pool::Postgres(pool))
    }

    #[cfg(not(feature = "postgres"))]
    async fn connect_postgres(_url: &str) -> Result<Self, DalError> {
        Err(DalError::UnsupportedUrl(
            "postgres support not compiled in (enable the `postgres` feature)".into(),
        ))
    }

    /// Returns the backend this pool is connected to.
    pub fn backend(&self) -> Backend {
        match self {
            #[cfg(feature = "sqlite")]
            Pool::Sqlite(_) => Backend::Sqlite,
            #[cfg(feature = "mysql")]
            Pool::MySql(_) => Backend::MySql,
            #[cfg(feature = "postgres")]
            Pool::Postgres(_) => Backend::Postgres,
        }
    }

    /// Close all connections in the pool.
    pub async fn close(&self) {
        match self {
            #[cfg(feature = "sqlite")]
            Pool::Sqlite(p) => p.pool.close(),
            #[cfg(feature = "mysql")]
            Pool::MySql(p) => p.close().await,
            #[cfg(feature = "postgres")]
            Pool::Postgres(p) => p.close().await,
        }
    }

    /// Execute a statement that does not return rows.
    ///
    /// The SQL is sent verbatim. To include caller- or user-supplied data, use
    /// [`Pool::execute_with`] with bind parameters instead of formatting the
    /// value into `sql`.
    pub async fn execute(&self, sql: &str) -> Result<ExecResult, DalError> {
        self.execute_with(sql, &[]).await
    }

    /// Execute a non-row-returning statement with bind parameters.
    ///
    /// Values travel to the database out-of-band and are never parsed as SQL,
    /// so this is the injection-safe way to include dynamic data.
    ///
    /// Placeholders are backend-native — `?` for SQLite and MySQL, `$1`/`$2`
    /// for Postgres. See [`Value`] for the full table.
    ///
    /// ```no_run
    /// # use ironroot_dal::{Pool, Value, DalError};
    /// # async fn demo(pool: &Pool, name: &str) -> Result<(), DalError> {
    /// pool.execute_with("DELETE FROM users WHERE name = ?", &[Value::from(name)])
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_with(&self, sql: &str, params: &[Value]) -> Result<ExecResult, DalError> {
        crate::placeholder::validate(sql, self.backend(), params.len())?;
        match self {
            #[cfg(feature = "sqlite")]
            Pool::Sqlite(p) => {
                let sql = sql.to_string();
                let params = params.to_vec();
                // `last_insert_rowid` is per-connection, so it must be read on
                // the same pooled connection that ran the INSERT.
                p.interact(move |conn| {
                    let rows = conn
                        .execute(&sql, rusqlite::params_from_iter(params.iter()))
                        .map_err(map_rusqlite)?;
                    Ok(ExecResult {
                        rows_affected: rows as u64,
                        last_insert_id: Some(conn.last_insert_rowid()),
                    })
                })
                .await
            }
            #[cfg(feature = "mysql")]
            Pool::MySql(p) => {
                let r = bind_params!(sqlx_core::query::query(AssertSqlSafe(sql)), params)
                    .execute(p)
                    .await?;
                Ok(ExecResult {
                    rows_affected: r.rows_affected(),
                    last_insert_id: Some(r.last_insert_id() as i64),
                })
            }
            #[cfg(feature = "postgres")]
            Pool::Postgres(p) => {
                let r = bind_params!(sqlx_core::query::query(AssertSqlSafe(sql)), params)
                    .execute(p)
                    .await?;
                Ok(ExecResult {
                    rows_affected: r.rows_affected(),
                    last_insert_id: None,
                })
            }
        }
    }

    /// Fetch all rows matching the given query.
    ///
    /// Prefer [`Pool::fetch_all_with`] whenever the query contains dynamic data.
    pub async fn fetch_all(&self, sql: &str) -> Result<Vec<Row>, DalError> {
        self.fetch_all_with(sql, &[]).await
    }

    /// Fetch all rows matching the given query, with bind parameters.
    pub async fn fetch_all_with(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>, DalError> {
        crate::placeholder::validate(sql, self.backend(), params.len())?;
        match self {
            #[cfg(feature = "sqlite")]
            Pool::Sqlite(p) => {
                let sql = sql.to_string();
                let params = params.to_vec();
                p.interact(move |conn| {
                    let mut stmt = conn.prepare(&sql).map_err(map_rusqlite)?;
                    let mut rows = stmt
                        .query(rusqlite::params_from_iter(params.iter()))
                        .map_err(map_rusqlite)?;
                    let mut out = Vec::new();
                    while let Some(row) = rows.next().map_err(map_rusqlite)? {
                        let r = crate::row::SqliteRow::from_row(row).map_err(map_rusqlite)?;
                        out.push(Row::from_sqlite(r));
                    }
                    Ok(out)
                })
                .await
            }
            #[cfg(feature = "mysql")]
            Pool::MySql(p) => {
                let rows = bind_params!(sqlx_core::query::query(AssertSqlSafe(sql)), params)
                    .fetch_all(p)
                    .await?;
                Ok(rows.into_iter().map(Row::from_mysql).collect())
            }
            #[cfg(feature = "postgres")]
            Pool::Postgres(p) => {
                let rows = bind_params!(sqlx_core::query::query(AssertSqlSafe(sql)), params)
                    .fetch_all(p)
                    .await?;
                Ok(rows.into_iter().map(Row::from_pg).collect())
            }
        }
    }

    /// Fetch at most one row.
    ///
    /// Prefer [`Pool::fetch_optional_with`] whenever the query contains
    /// dynamic data.
    pub async fn fetch_optional(&self, sql: &str) -> Result<Option<Row>, DalError> {
        self.fetch_optional_with(sql, &[]).await
    }

    /// Fetch at most one row, with bind parameters.
    pub async fn fetch_optional_with(
        &self,
        sql: &str,
        params: &[Value],
    ) -> Result<Option<Row>, DalError> {
        crate::placeholder::validate(sql, self.backend(), params.len())?;
        match self {
            #[cfg(feature = "sqlite")]
            Pool::Sqlite(p) => {
                let sql = sql.to_string();
                let params = params.to_vec();
                p.interact(move |conn| {
                    let mut stmt = conn.prepare(&sql).map_err(map_rusqlite)?;
                    let mut rows = stmt
                        .query(rusqlite::params_from_iter(params.iter()))
                        .map_err(map_rusqlite)?;
                    let opt = match rows.next().map_err(map_rusqlite)? {
                        Some(row) => {
                            Some(crate::row::SqliteRow::from_row(row).map_err(map_rusqlite)?)
                        }
                        None => None,
                    };
                    Ok(opt.map(Row::from_sqlite))
                })
                .await
            }
            #[cfg(feature = "mysql")]
            Pool::MySql(p) => Ok(
                bind_params!(sqlx_core::query::query(AssertSqlSafe(sql)), params)
                    .fetch_optional(p)
                    .await?
                    .map(Row::from_mysql),
            ),
            #[cfg(feature = "postgres")]
            Pool::Postgres(p) => Ok(bind_params!(
                sqlx_core::query::query(AssertSqlSafe(sql)),
                params
            )
            .fetch_optional(p)
            .await?
            .map(Row::from_pg)),
        }
    }
}

#[async_trait]
impl DalPool for Pool {
    fn backend(&self) -> Backend {
        Pool::backend(self)
    }

    async fn execute(&self, sql: &str) -> Result<ExecResult, DalError> {
        Pool::execute(self, sql).await
    }

    async fn execute_with(&self, sql: &str, params: &[Value]) -> Result<ExecResult, DalError> {
        Pool::execute_with(self, sql, params).await
    }

    async fn fetch_all(&self, sql: &str) -> Result<Vec<Row>, DalError> {
        Pool::fetch_all(self, sql).await
    }

    async fn fetch_all_with(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>, DalError> {
        Pool::fetch_all_with(self, sql, params).await
    }

    async fn fetch_optional(&self, sql: &str) -> Result<Option<Row>, DalError> {
        Pool::fetch_optional(self, sql).await
    }

    async fn fetch_optional_with(
        &self,
        sql: &str,
        params: &[Value],
    ) -> Result<Option<Row>, DalError> {
        Pool::fetch_optional_with(self, sql, params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn bound_params_roundtrip_all_value_kinds() {
        let pool = Pool::connect("sqlite::memory:").await.unwrap();
        pool.execute(
            "CREATE TABLE v (id INTEGER PRIMARY KEY, t TEXT, i INTEGER, r REAL, b BLOB, n TEXT)",
        )
        .await
        .unwrap();

        pool.execute_with(
            "INSERT INTO v (t, i, r, b, n) VALUES (?, ?, ?, ?, ?)",
            &[
                Value::from("Ada"),
                Value::from(42_i64),
                Value::from(1.5_f64),
                Value::from(&b"xy"[..]),
                Value::Null,
            ],
        )
        .await
        .unwrap();

        let row = pool
            .fetch_optional_with("SELECT t, i, r FROM v WHERE i = ?", &[Value::from(42_i64)])
            .await
            .unwrap()
            .expect("row present");
        assert_eq!(row.try_get_string("t").unwrap(), "Ada");
        assert_eq!(row.try_get_i64("i").unwrap(), 42);
        assert!((row.try_get_f64("r").unwrap() - 1.5).abs() < f64::EPSILON);
    }

    /// The whole point of bind parameters: a value containing SQL syntax must
    /// be treated as data. With string interpolation this input would end the
    /// statement and drop the table.
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn bound_params_neutralise_injection_attempts() {
        let pool = Pool::connect("sqlite::memory:").await.unwrap();
        pool.execute("CREATE TABLE u (id INTEGER PRIMARY KEY, name TEXT)")
            .await
            .unwrap();

        let hostile = "'; DROP TABLE u; --";
        pool.execute_with("INSERT INTO u (name) VALUES (?)", &[Value::from(hostile)])
            .await
            .unwrap();

        // The table still exists, and the payload was stored verbatim as data.
        let row = pool
            .fetch_optional_with("SELECT name FROM u WHERE name = ?", &[Value::from(hostile)])
            .await
            .unwrap()
            .expect("hostile string stored as data");
        assert_eq!(row.try_get_string("name").unwrap(), hostile);

        let all = pool.fetch_all("SELECT id FROM u").await.unwrap();
        assert_eq!(all.len(), 1, "table survived the injection attempt");
    }

    /// Every connection to `:memory:` opens its *own* private database, so a
    /// pool larger than one would hand out connections to different, empty
    /// databases. Writes must stay visible to subsequent reads.
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn in_memory_pool_keeps_one_shared_database() {
        let pool = Pool::connect("sqlite::memory:").await.unwrap();
        pool.execute("CREATE TABLE m (id INTEGER PRIMARY KEY, v TEXT)")
            .await
            .unwrap();
        pool.execute_with("INSERT INTO m (v) VALUES (?)", &[Value::from("kept")])
            .await
            .unwrap();

        // Several checkouts in a row must all observe the same database.
        for _ in 0..5 {
            let rows = pool.fetch_all("SELECT v FROM m").await.unwrap();
            assert_eq!(rows.len(), 1, "in-memory database was not shared");
            assert_eq!(rows[0].try_get_string("v").unwrap(), "kept");
        }
    }

    /// A file-backed pool should genuinely overlap work rather than serialising
    /// every query behind a single connection.
    #[cfg(feature = "sqlite")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn file_backed_pool_serves_concurrent_queries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pool.db");
        let url = format!("sqlite:{}", path.display());

        let pool = Pool::connect(&url).await.unwrap();
        pool.execute("CREATE TABLE c (id INTEGER PRIMARY KEY, v INTEGER)")
            .await
            .unwrap();
        for i in 0..20_i64 {
            pool.execute_with("INSERT INTO c (v) VALUES (?)", &[Value::from(i)])
                .await
                .unwrap();
        }

        let mut handles = Vec::new();
        for _ in 0..16 {
            let pool = pool.clone();
            handles.push(tokio::spawn(async move {
                let rows = pool.fetch_all("SELECT v FROM c").await.unwrap();
                rows.len()
            }));
        }
        for h in handles {
            assert_eq!(h.await.unwrap(), 20);
        }

        // Concurrent writers must not fail with SQLITE_BUSY either.
        let mut writers = Vec::new();
        for i in 100..116_i64 {
            let pool = pool.clone();
            writers.push(tokio::spawn(async move {
                pool.execute_with("INSERT INTO c (v) VALUES (?)", &[Value::from(i)])
                    .await
            }));
        }
        for w in writers {
            w.await.unwrap().expect("concurrent write should not fail");
        }

        let rows = pool.fetch_all("SELECT v FROM c").await.unwrap();
        assert_eq!(rows.len(), 36);
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn placeholder_mismatch_is_caught_before_the_driver() {
        let pool = Pool::connect("sqlite::memory:").await.unwrap();
        pool.execute("CREATE TABLE p (id INTEGER PRIMARY KEY, a TEXT, b TEXT)")
            .await
            .unwrap();

        // Two placeholders, one parameter.
        let err = pool
            .fetch_all_with("SELECT * FROM p WHERE a = ? AND b = ?", &[Value::from("x")])
            .await
            .unwrap_err();
        assert!(matches!(err, DalError::Placeholder(_)), "{err}");

        // Postgres-style placeholders against SQLite.
        let err = pool
            .fetch_all_with("SELECT * FROM p WHERE a = $1", &[Value::from("x")])
            .await
            .unwrap_err();
        assert!(matches!(err, DalError::Placeholder(_)), "{err}");

        // A `?` inside a literal is not a placeholder, so this is well-formed.
        pool.fetch_all_with(
            "SELECT * FROM p WHERE a = 'why?' AND b = ?",
            &[Value::from("x")],
        )
        .await
        .expect("literal `?` must not be counted");
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn null_binding_matches_is_null() {
        let pool = Pool::connect("sqlite::memory:").await.unwrap();
        pool.execute("CREATE TABLE n (id INTEGER PRIMARY KEY, note TEXT)")
            .await
            .unwrap();
        pool.execute_with(
            "INSERT INTO n (note) VALUES (?)",
            &[Value::from(None::<String>)],
        )
        .await
        .unwrap();

        let rows = pool
            .fetch_all("SELECT id FROM n WHERE note IS NULL")
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn parses_backend_from_url_scheme() {
        assert_eq!(
            Backend::from_url("sqlite::memory:").unwrap(),
            Backend::Sqlite
        );
        assert_eq!(
            Backend::from_url("sqlite:///tmp/test.db").unwrap(),
            Backend::Sqlite
        );
        assert_eq!(
            Backend::from_url("mysql://u:p@h:3306/db").unwrap(),
            Backend::MySql
        );
        assert_eq!(
            Backend::from_url("mariadb://u:p@h:3306/db").unwrap(),
            Backend::MySql
        );
        assert_eq!(
            Backend::from_url("postgres://u@h/db").unwrap(),
            Backend::Postgres
        );
        assert_eq!(
            Backend::from_url("postgresql://u@h/db").unwrap(),
            Backend::Postgres
        );
        assert!(Backend::from_url("redis://h").is_err());
        assert!(Backend::from_url("not a url").is_err());
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn sqlite_roundtrip_in_memory() {
        let pool = Pool::connect("sqlite::memory:").await.unwrap();
        assert_eq!(pool.backend(), Backend::Sqlite);

        pool.execute("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .await
            .unwrap();

        let r = pool
            .execute("INSERT INTO t (name) VALUES ('Ada'), ('Bob')")
            .await
            .unwrap();
        assert_eq!(r.rows_affected, 2);

        let rows = pool
            .fetch_all("SELECT id, name FROM t ORDER BY id")
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].try_get_string("name").unwrap(), "Ada");
        assert_eq!(rows[1].try_get_string("name").unwrap(), "Bob");

        let one = pool
            .fetch_optional("SELECT id FROM t WHERE name = 'Ada'")
            .await
            .unwrap();
        assert!(one.is_some());

        let none = pool
            .fetch_optional("SELECT id FROM t WHERE name = 'nope'")
            .await
            .unwrap();
        assert!(none.is_none());
    }
}
