//! # ironroot-dal-hiqlite
//!
//! Hiqlite integration for the IronRoot DAL family.
//!
//! [Hiqlite](https://crates.io/crates/hiqlite) is a Raft-backed embedded
//! SQLite that can run as a highly-available cluster. This crate exposes it
//! through a [`HiqlitePool`] wrapper with an `execute` shape matching
//! `ironroot-dal`'s pool API, while letting power users drop down to the
//! underlying [`hiqlite::Client`] for full read APIs.
//!
//! ## Why a separate crate (not a feature on `ironroot-dal`)?
//!
//! `ironroot-dal`'s SQLite backend is built on `rusqlite` — the same crate
//! hiqlite uses — so the two can coexist in one Cargo workspace. Hiqlite
//! lives in its own crate so callers that only need the Raft-backed
//! variant don't pull sqlx and the MySQL / PostgreSQL transitive deps.
//!
//! This crate intentionally does **not** depend on `ironroot-dal`; the
//! small [`DalError`] / [`ExecResult`] shapes are mirrored locally and
//! are API-compatible with their counterparts there.
//!
//! ## Example
//!
//! ```ignore
//! use ironroot_dal_hiqlite::{HiqlitePool, hiqlite};
//! use std::sync::Arc;
//!
//! let client: hiqlite::Client = /* start_node_with_cache(...) */;
//! let pool = HiqlitePool::new(Arc::new(client));
//! pool.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)").await?;
//! ```

use std::sync::Arc;

use thiserror::Error;

/// Re-export of the hiqlite crate so downstream code only needs one dependency.
pub use hiqlite;

/// Error returned by [`HiqlitePool`]. API-compatible with
/// `ironroot_dal::DalError`'s `Database` variant — wrap via `.to_string()`
/// when bridging.
#[derive(Debug, Error)]
pub enum DalError {
    /// The underlying database driver returned an error.
    #[error("database error: {0}")]
    Database(String),
}

/// Outcome of a non-row-returning statement. Mirrors
/// `ironroot_dal::ExecResult`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExecResult {
    /// Number of rows affected.
    pub rows_affected: u64,
    /// Last insert id, when the backend reports one. Hiqlite does not
    /// surface this through the high-level `execute` API.
    pub last_insert_id: Option<i64>,
}

/// Wrapper around a [`hiqlite::Client`] exposing a familiar `execute` /
/// `client` surface.
#[derive(Clone)]
pub struct HiqlitePool {
    client: Arc<hiqlite::Client>,
}

impl HiqlitePool {
    /// Wrap an already-constructed hiqlite client.
    pub fn new(client: Arc<hiqlite::Client>) -> Self {
        Self { client }
    }

    /// Borrow the underlying client for advanced operations
    /// (`query_raw`, `query_map`, transactions, batch, cache, etc.).
    pub fn client(&self) -> &hiqlite::Client {
        &self.client
    }

    /// Execute a statement that does not return rows.
    ///
    /// `hiqlite::Client::execute` requires `Into<Cow<'static, str>>`, so the
    /// SQL is owned (`String`) here rather than borrowed.
    pub async fn execute(&self, sql: &str) -> Result<ExecResult, DalError> {
        tracing::trace!(target: "ironroot_dal_hiqlite", sql, "execute");
        let rows = self
            .client
            .execute(sql.to_string(), Vec::new())
            .await
            .map_err(map_err)?;
        Ok(ExecResult {
            rows_affected: rows as u64,
            last_insert_id: None,
        })
    }

    /// Execute a parameterised statement.
    ///
    /// Pass parameters as a `hiqlite::Params` (i.e. `Vec<hiqlite::Param>`).
    /// If you've enabled hiqlite's `macros` feature in your own crate, you
    /// can also use `hiqlite::params!` to build that vector ergonomically.
    pub async fn execute_params(
        &self,
        sql: &str,
        params: hiqlite::Params,
    ) -> Result<ExecResult, DalError> {
        let rows = self
            .client
            .execute(sql.to_string(), params)
            .await
            .map_err(map_err)?;
        Ok(ExecResult {
            rows_affected: rows as u64,
            last_insert_id: None,
        })
    }
}

fn map_err(err: hiqlite::Error) -> DalError {
    DalError::Database(format!("hiqlite: {err}"))
}
