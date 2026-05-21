//! Backend-agnostic row wrapper.
//!
//! [`Row`] hides the concrete sqlx row types so consumers can write code that
//! works against any supported backend.

use crate::DalError;

/// A row returned from any supported backend.
///
/// Use the `try_get_*` accessors to extract typed columns by name.
pub enum Row {
    #[cfg(feature = "sqlite")]
    /// SQLite row.
    Sqlite(sqlx::sqlite::SqliteRow),
    #[cfg(feature = "mysql")]
    /// MySQL row.
    MySql(sqlx::mysql::MySqlRow),
    #[cfg(feature = "postgres")]
    /// PostgreSQL row.
    Postgres(sqlx::postgres::PgRow),
}

impl std::fmt::Debug for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let backend = match self {
            #[cfg(feature = "sqlite")]
            Row::Sqlite(_) => "Sqlite",
            #[cfg(feature = "mysql")]
            Row::MySql(_) => "MySql",
            #[cfg(feature = "postgres")]
            Row::Postgres(_) => "Postgres",
        };
        f.debug_struct("Row").field("backend", &backend).finish()
    }
}

impl Row {
    #[cfg(feature = "sqlite")]
    pub(crate) fn from_sqlite(r: sqlx::sqlite::SqliteRow) -> Self {
        Row::Sqlite(r)
    }

    #[cfg(feature = "mysql")]
    pub(crate) fn from_mysql(r: sqlx::mysql::MySqlRow) -> Self {
        Row::MySql(r)
    }

    #[cfg(feature = "postgres")]
    pub(crate) fn from_pg(r: sqlx::postgres::PgRow) -> Self {
        Row::Postgres(r)
    }

    /// Try to read an `i64`-compatible column by name.
    pub fn try_get_i64(&self, column: &str) -> Result<i64, DalError> {
        use sqlx::Row as _;
        match self {
            #[cfg(feature = "sqlite")]
            Row::Sqlite(r) => r.try_get::<i64, _>(column).map_err(|e| decode(column, e)),
            #[cfg(feature = "mysql")]
            Row::MySql(r) => r.try_get::<i64, _>(column).map_err(|e| decode(column, e)),
            #[cfg(feature = "postgres")]
            Row::Postgres(r) => r.try_get::<i64, _>(column).map_err(|e| decode(column, e)),
        }
    }

    /// Try to read a `String` column by name.
    pub fn try_get_string(&self, column: &str) -> Result<String, DalError> {
        use sqlx::Row as _;
        match self {
            #[cfg(feature = "sqlite")]
            Row::Sqlite(r) => r.try_get::<String, _>(column).map_err(|e| decode(column, e)),
            #[cfg(feature = "mysql")]
            Row::MySql(r) => r.try_get::<String, _>(column).map_err(|e| decode(column, e)),
            #[cfg(feature = "postgres")]
            Row::Postgres(r) => r.try_get::<String, _>(column).map_err(|e| decode(column, e)),
        }
    }

    /// Try to read a `bool` column by name.
    pub fn try_get_bool(&self, column: &str) -> Result<bool, DalError> {
        use sqlx::Row as _;
        match self {
            #[cfg(feature = "sqlite")]
            Row::Sqlite(r) => r.try_get::<bool, _>(column).map_err(|e| decode(column, e)),
            #[cfg(feature = "mysql")]
            Row::MySql(r) => r.try_get::<bool, _>(column).map_err(|e| decode(column, e)),
            #[cfg(feature = "postgres")]
            Row::Postgres(r) => r.try_get::<bool, _>(column).map_err(|e| decode(column, e)),
        }
    }

    /// Try to read an `f64` column by name.
    pub fn try_get_f64(&self, column: &str) -> Result<f64, DalError> {
        use sqlx::Row as _;
        match self {
            #[cfg(feature = "sqlite")]
            Row::Sqlite(r) => r.try_get::<f64, _>(column).map_err(|e| decode(column, e)),
            #[cfg(feature = "mysql")]
            Row::MySql(r) => r.try_get::<f64, _>(column).map_err(|e| decode(column, e)),
            #[cfg(feature = "postgres")]
            Row::Postgres(r) => r.try_get::<f64, _>(column).map_err(|e| decode(column, e)),
        }
    }
}

fn decode(column: &str, err: sqlx::Error) -> DalError {
    DalError::Decode {
        column: column.to_string(),
        message: err.to_string(),
    }
}
