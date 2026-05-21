//! Backend-routed connection pool.

use async_trait::async_trait;

use crate::{row::Row, DalError, DalPool, ExecResult};

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

/// A runtime-routed connection pool that dispatches to the appropriate
/// `sqlx` backend.
///
/// Construct with [`Pool::connect`]; the URL scheme determines the backend.
#[derive(Debug, Clone)]
pub enum Pool {
    /// SQLite-backed pool.
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::SqlitePool),
    /// MySQL-backed pool.
    #[cfg(feature = "mysql")]
    MySql(sqlx::MySqlPool),
    /// PostgreSQL-backed pool.
    #[cfg(feature = "postgres")]
    Postgres(sqlx::PgPool),
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
        let pool = sqlx::SqlitePool::connect(url).await?;
        Ok(Pool::Sqlite(pool))
    }

    #[cfg(not(feature = "sqlite"))]
    async fn connect_sqlite(_url: &str) -> Result<Self, DalError> {
        Err(DalError::UnsupportedUrl(
            "sqlite support not compiled in (enable the `sqlite` feature)".into(),
        ))
    }

    #[cfg(feature = "mysql")]
    async fn connect_mysql(url: &str) -> Result<Self, DalError> {
        let pool = sqlx::MySqlPool::connect(url).await?;
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
        let pool = sqlx::PgPool::connect(url).await?;
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
            Pool::Sqlite(p) => p.close().await,
            #[cfg(feature = "mysql")]
            Pool::MySql(p) => p.close().await,
            #[cfg(feature = "postgres")]
            Pool::Postgres(p) => p.close().await,
        }
    }

    /// Execute a statement that does not return rows.
    pub async fn execute(&self, sql: &str) -> Result<ExecResult, DalError> {
        match self {
            #[cfg(feature = "sqlite")]
            Pool::Sqlite(p) => {
                let r = sqlx::query(sql).execute(p).await?;
                Ok(ExecResult {
                    rows_affected: r.rows_affected(),
                    last_insert_id: Some(r.last_insert_rowid()),
                })
            }
            #[cfg(feature = "mysql")]
            Pool::MySql(p) => {
                let r = sqlx::query(sql).execute(p).await?;
                Ok(ExecResult {
                    rows_affected: r.rows_affected(),
                    last_insert_id: Some(r.last_insert_id() as i64),
                })
            }
            #[cfg(feature = "postgres")]
            Pool::Postgres(p) => {
                let r = sqlx::query(sql).execute(p).await?;
                Ok(ExecResult {
                    rows_affected: r.rows_affected(),
                    last_insert_id: None,
                })
            }
        }
    }

    /// Fetch all rows matching the given query.
    pub async fn fetch_all(&self, sql: &str) -> Result<Vec<Row>, DalError> {
        match self {
            #[cfg(feature = "sqlite")]
            Pool::Sqlite(p) => {
                let rows = sqlx::query(sql).fetch_all(p).await?;
                Ok(rows.into_iter().map(Row::from_sqlite).collect())
            }
            #[cfg(feature = "mysql")]
            Pool::MySql(p) => {
                let rows = sqlx::query(sql).fetch_all(p).await?;
                Ok(rows.into_iter().map(Row::from_mysql).collect())
            }
            #[cfg(feature = "postgres")]
            Pool::Postgres(p) => {
                let rows = sqlx::query(sql).fetch_all(p).await?;
                Ok(rows.into_iter().map(Row::from_pg).collect())
            }
        }
    }

    /// Fetch at most one row.
    pub async fn fetch_optional(&self, sql: &str) -> Result<Option<Row>, DalError> {
        match self {
            #[cfg(feature = "sqlite")]
            Pool::Sqlite(p) => Ok(sqlx::query(sql).fetch_optional(p).await?.map(Row::from_sqlite)),
            #[cfg(feature = "mysql")]
            Pool::MySql(p) => Ok(sqlx::query(sql).fetch_optional(p).await?.map(Row::from_mysql)),
            #[cfg(feature = "postgres")]
            Pool::Postgres(p) => Ok(sqlx::query(sql).fetch_optional(p).await?.map(Row::from_pg)),
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

    async fn fetch_all(&self, sql: &str) -> Result<Vec<Row>, DalError> {
        Pool::fetch_all(self, sql).await
    }

    async fn fetch_optional(&self, sql: &str) -> Result<Option<Row>, DalError> {
        Pool::fetch_optional(self, sql).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_backend_from_url_scheme() {
        assert_eq!(Backend::from_url("sqlite::memory:").unwrap(), Backend::Sqlite);
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

        let rows = pool.fetch_all("SELECT id, name FROM t ORDER BY id").await.unwrap();
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
