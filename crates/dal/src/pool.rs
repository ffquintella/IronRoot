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

/// Shared rusqlite connection wrapped for async use via `spawn_blocking`.
#[cfg(feature = "sqlite")]
#[derive(Clone)]
pub struct SqlitePool {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

#[cfg(feature = "sqlite")]
impl std::fmt::Debug for SqlitePool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqlitePool").finish_non_exhaustive()
    }
}

#[cfg(feature = "sqlite")]
impl SqlitePool {
    /// Open from a connection URL (`sqlite::memory:`, `sqlite:path.db`, `sqlite:///abs/path`).
    pub fn open(url: &str) -> Result<Self, DalError> {
        let conn = open_sqlite(url)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[cfg(feature = "sqlite")]
fn open_sqlite(url: &str) -> Result<rusqlite::Connection, DalError> {
    let rest = url
        .strip_prefix("sqlite:")
        .ok_or_else(|| DalError::UnsupportedUrl(url.to_string()))?;
    let conn = if rest == ":memory:" || rest.is_empty() {
        rusqlite::Connection::open_in_memory()
    } else {
        let path = rest.trim_start_matches("//");
        rusqlite::Connection::open(path)
    };
    conn.map_err(|e| DalError::Database(e.to_string()))
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
            Pool::Sqlite(_) => { /* rusqlite connection drops with the Arc */ }
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
        match self {
            #[cfg(feature = "sqlite")]
            Pool::Sqlite(p) => {
                let conn = p.conn.clone();
                let sql = sql.to_string();
                let params = params.to_vec();
                tokio::task::spawn_blocking(move || {
                    let conn = conn.lock().expect("sqlite mutex poisoned");
                    let rows = conn
                        .execute(&sql, rusqlite::params_from_iter(params.iter()))
                        .map_err(map_rusqlite)?;
                    let last_id = conn.last_insert_rowid();
                    Ok::<_, DalError>(ExecResult {
                        rows_affected: rows as u64,
                        last_insert_id: Some(last_id),
                    })
                })
                .await
                .map_err(|e| DalError::Database(e.to_string()))?
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
        match self {
            #[cfg(feature = "sqlite")]
            Pool::Sqlite(p) => {
                let conn = p.conn.clone();
                let sql = sql.to_string();
                let params = params.to_vec();
                tokio::task::spawn_blocking(move || {
                    let conn = conn.lock().expect("sqlite mutex poisoned");
                    let mut stmt = conn.prepare(&sql).map_err(map_rusqlite)?;
                    let mut rows = stmt
                        .query(rusqlite::params_from_iter(params.iter()))
                        .map_err(map_rusqlite)?;
                    let mut out = Vec::new();
                    while let Some(row) = rows.next().map_err(map_rusqlite)? {
                        let r = crate::row::SqliteRow::from_row(row).map_err(map_rusqlite)?;
                        out.push(Row::from_sqlite(r));
                    }
                    Ok::<_, DalError>(out)
                })
                .await
                .map_err(|e| DalError::Database(e.to_string()))?
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
        match self {
            #[cfg(feature = "sqlite")]
            Pool::Sqlite(p) => {
                let conn = p.conn.clone();
                let sql = sql.to_string();
                let params = params.to_vec();
                tokio::task::spawn_blocking(move || {
                    let conn = conn.lock().expect("sqlite mutex poisoned");
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
                    Ok::<_, DalError>(opt.map(Row::from_sqlite))
                })
                .await
                .map_err(|e| DalError::Database(e.to_string()))?
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
