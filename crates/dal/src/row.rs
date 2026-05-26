//! Backend-agnostic row wrapper.
//!
//! [`Row`] hides the concrete driver row types so consumers can write code
//! that works against any supported backend.

use crate::DalError;

/// A row returned from any supported backend.
///
/// Use the `try_get_*` accessors to extract typed columns by name.
pub enum Row {
    /// SQLite row (materialised — rusqlite rows borrow from a statement).
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteRow),
    /// MySQL row.
    #[cfg(feature = "mysql")]
    MySql(sqlx_mysql::MySqlRow),
    /// PostgreSQL row.
    #[cfg(feature = "postgres")]
    Postgres(sqlx_postgres::PgRow),
}

#[cfg(feature = "sqlite")]
#[derive(Debug, Clone)]
pub struct SqliteRow {
    pub(crate) values: Vec<(String, rusqlite::types::Value)>,
}

#[cfg(feature = "sqlite")]
impl SqliteRow {
    pub(crate) fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let stmt = row.as_ref();
        let count = stmt.column_count();
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            let name = stmt.column_name(i)?.to_string();
            let value: rusqlite::types::Value = row.get(i)?;
            values.push((name, value));
        }
        Ok(SqliteRow { values })
    }

    fn lookup(&self, column: &str) -> Result<&rusqlite::types::Value, DalError> {
        self.values
            .iter()
            .find(|(n, _)| n == column)
            .map(|(_, v)| v)
            .ok_or_else(|| DalError::Decode {
                column: column.to_string(),
                message: "column not found".into(),
            })
    }
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
    pub(crate) fn from_sqlite(r: SqliteRow) -> Self {
        Row::Sqlite(r)
    }

    #[cfg(feature = "mysql")]
    pub(crate) fn from_mysql(r: sqlx_mysql::MySqlRow) -> Self {
        Row::MySql(r)
    }

    #[cfg(feature = "postgres")]
    pub(crate) fn from_pg(r: sqlx_postgres::PgRow) -> Self {
        Row::Postgres(r)
    }

    /// Try to read an `i64`-compatible column by name.
    pub fn try_get_i64(&self, column: &str) -> Result<i64, DalError> {
        #[cfg(any(feature = "mysql", feature = "postgres"))]
        use sqlx_core::row::Row as _;
        match self {
            #[cfg(feature = "sqlite")]
            Row::Sqlite(r) => match r.lookup(column)? {
                rusqlite::types::Value::Integer(i) => Ok(*i),
                rusqlite::types::Value::Real(f) => Ok(*f as i64),
                other => Err(DalError::Decode {
                    column: column.to_string(),
                    message: format!("expected integer, got {other:?}"),
                }),
            },
            #[cfg(feature = "mysql")]
            Row::MySql(r) => r.try_get::<i64, _>(column).map_err(|e| decode_sqlx(column, e)),
            #[cfg(feature = "postgres")]
            Row::Postgres(r) => r.try_get::<i64, _>(column).map_err(|e| decode_sqlx(column, e)),
        }
    }

    /// Try to read a `String` column by name.
    pub fn try_get_string(&self, column: &str) -> Result<String, DalError> {
        #[cfg(any(feature = "mysql", feature = "postgres"))]
        use sqlx_core::row::Row as _;
        match self {
            #[cfg(feature = "sqlite")]
            Row::Sqlite(r) => match r.lookup(column)? {
                rusqlite::types::Value::Text(s) => Ok(s.clone()),
                other => Err(DalError::Decode {
                    column: column.to_string(),
                    message: format!("expected text, got {other:?}"),
                }),
            },
            #[cfg(feature = "mysql")]
            Row::MySql(r) => r.try_get::<String, _>(column).map_err(|e| decode_sqlx(column, e)),
            #[cfg(feature = "postgres")]
            Row::Postgres(r) => r.try_get::<String, _>(column).map_err(|e| decode_sqlx(column, e)),
        }
    }

    /// Try to read a `bool` column by name.
    pub fn try_get_bool(&self, column: &str) -> Result<bool, DalError> {
        #[cfg(any(feature = "mysql", feature = "postgres"))]
        use sqlx_core::row::Row as _;
        match self {
            #[cfg(feature = "sqlite")]
            Row::Sqlite(r) => match r.lookup(column)? {
                rusqlite::types::Value::Integer(i) => Ok(*i != 0),
                other => Err(DalError::Decode {
                    column: column.to_string(),
                    message: format!("expected integer-as-bool, got {other:?}"),
                }),
            },
            #[cfg(feature = "mysql")]
            Row::MySql(r) => r.try_get::<bool, _>(column).map_err(|e| decode_sqlx(column, e)),
            #[cfg(feature = "postgres")]
            Row::Postgres(r) => r.try_get::<bool, _>(column).map_err(|e| decode_sqlx(column, e)),
        }
    }

    /// Try to read an `f64` column by name.
    pub fn try_get_f64(&self, column: &str) -> Result<f64, DalError> {
        #[cfg(any(feature = "mysql", feature = "postgres"))]
        use sqlx_core::row::Row as _;
        match self {
            #[cfg(feature = "sqlite")]
            Row::Sqlite(r) => match r.lookup(column)? {
                rusqlite::types::Value::Real(f) => Ok(*f),
                rusqlite::types::Value::Integer(i) => Ok(*i as f64),
                other => Err(DalError::Decode {
                    column: column.to_string(),
                    message: format!("expected real, got {other:?}"),
                }),
            },
            #[cfg(feature = "mysql")]
            Row::MySql(r) => r.try_get::<f64, _>(column).map_err(|e| decode_sqlx(column, e)),
            #[cfg(feature = "postgres")]
            Row::Postgres(r) => r.try_get::<f64, _>(column).map_err(|e| decode_sqlx(column, e)),
        }
    }
}

#[cfg(any(feature = "mysql", feature = "postgres"))]
fn decode_sqlx(column: &str, err: sqlx_core::error::Error) -> DalError {
    DalError::Decode {
        column: column.to_string(),
        message: err.to_string(),
    }
}
