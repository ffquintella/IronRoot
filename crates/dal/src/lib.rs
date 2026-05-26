//! # ironroot-dal
//!
//! Data Access Layer for the IronRoot framework.
//!
//! `ironroot-dal` wraps [`sqlx`] with a runtime-routed connection pool that
//! transparently supports **SQLite** and **MySQL** (and optionally PostgreSQL
//! via the `postgres` feature). Application code can pick the backend at
//! startup from a connection-string scheme — no recompilation required.
//!
//! ## Quick start
//!
//! ```no_run
//! use ironroot_dal::{Pool, DalError};
//!
//! # async fn run() -> Result<(), DalError> {
//! // Auto-detected from the URL scheme:
//! let pool = Pool::connect("sqlite::memory:").await?;
//! // or:
//! // let pool = Pool::connect("mysql://user:pw@localhost:3306/app").await?;
//!
//! pool.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").await?;
//! pool.execute("INSERT INTO users (id, name) VALUES (1, 'Ada')").await?;
//! let rows = pool.fetch_all("SELECT id, name FROM users").await?;
//! assert_eq!(rows.len(), 1);
//! # Ok(())
//! # }
//! ```
//!
//! ## Backend selection
//!
//! The backend is chosen from the URL scheme:
//!
//! | Scheme | Backend |
//! |---|---|
//! | `sqlite:` | SQLite |
//! | `mysql:`  | MySQL / MariaDB |
//! | `postgres:` / `postgresql:` | PostgreSQL (feature-gated) |
//!
//! ## Repositories
//!
//! Higher-level access is provided through the [`Repository`] trait — implement
//! it on your domain types to get a uniform CRUD-style interface, regardless
//! of the backend in use.

use std::fmt;

use async_trait::async_trait;
use thiserror::Error;

mod pool;
mod repository;
mod row;

pub use pool::{Backend, Pool};
pub use repository::Repository;
pub use row::Row;

/// Top-level error type returned by all DAL operations.
#[derive(Debug, Error)]
pub enum DalError {
    /// The provided connection URL could not be parsed or its scheme is not
    /// supported by the enabled features.
    #[error("unsupported or malformed database URL: {0}")]
    UnsupportedUrl(String),

    /// The underlying database driver returned an error.
    #[error("database error: {0}")]
    Database(String),

    /// A row could not be decoded into the requested Rust type.
    #[error("failed to decode column {column:?}: {message}")]
    Decode {
        /// Name of the column that failed to decode.
        column: String,
        /// Underlying error message.
        message: String,
    },

    /// The requested entity was not found.
    #[error("not found")]
    NotFound,
}

impl From<sqlx_core::error::Error> for DalError {
    fn from(value: sqlx_core::error::Error) -> Self {
        match value {
            sqlx_core::error::Error::RowNotFound => DalError::NotFound,
            other => DalError::Database(other.to_string()),
        }
    }
}

/// A statement that does not return rows (INSERT, UPDATE, DELETE, DDL).
///
/// Returned by [`Pool::execute`] so callers can inspect how many rows were
/// affected.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExecResult {
    /// Number of rows affected by the statement.
    pub rows_affected: u64,
    /// Last insert id, if the backend reports one (MySQL/SQLite).
    pub last_insert_id: Option<i64>,
}

impl fmt::Display for ExecResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rows_affected={}", self.rows_affected)?;
        if let Some(id) = self.last_insert_id {
            write!(f, ", last_insert_id={}", id)?;
        }
        Ok(())
    }
}

/// Convenience trait so callers can write `pool.is_sqlite()` style checks.
#[async_trait]
pub trait DalPool: Send + Sync {
    /// Returns the backend the pool is connected to.
    fn backend(&self) -> Backend;

    /// Execute a statement that does not return rows.
    async fn execute(&self, sql: &str) -> Result<ExecResult, DalError>;

    /// Fetch all rows matching the query.
    async fn fetch_all(&self, sql: &str) -> Result<Vec<Row>, DalError>;

    /// Fetch at most one row.
    async fn fetch_optional(&self, sql: &str) -> Result<Option<Row>, DalError>;
}
