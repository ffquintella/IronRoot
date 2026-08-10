//! Bind-parameter values.
//!
//! [`Value`] is the backend-neutral parameter type used by the `*_with`
//! query methods on [`Pool`](crate::Pool). It exists so callers never have to
//! interpolate user data into SQL text: the value travels to the database
//! out-of-band, as a bound parameter, and is never parsed as SQL.
//!
//! # Placeholder syntax differs by backend
//!
//! IronRoot does **not** rewrite your SQL, so use the placeholder syntax your
//! backend expects:
//!
//! | Backend  | Placeholder |
//! |----------|-------------|
//! | SQLite   | `?`         |
//! | MySQL    | `?`         |
//! | Postgres | `$1`, `$2`  |
//!
//! If you need one query string to run on every backend, keep the SQL in the
//! repository implementation for that backend rather than sharing it.

/// A value bound to a query parameter.
///
/// Construct these with [`From`] rather than naming variants directly:
///
/// ```
/// use ironroot_dal::Value;
///
/// let params = [Value::from(42_i64), Value::from("Ada"), Value::Null];
/// assert_eq!(params.len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// SQL `NULL`.
    Null,
    /// Boolean. Stored as an integer on SQLite, which has no native boolean.
    Bool(bool),
    /// Signed 64-bit integer.
    Integer(i64),
    /// Double-precision float.
    Real(f64),
    /// UTF-8 text.
    Text(String),
    /// Raw bytes (`BLOB` / `BYTEA`).
    Blob(Vec<u8>),
}

macro_rules! from_int {
    ($($t:ty),*) => {$(
        impl From<$t> for Value {
            fn from(v: $t) -> Self {
                Value::Integer(i64::from(v))
            }
        }
    )*};
}
from_int!(i8, i16, i32, i64, u8, u16, u32);

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Real(f64::from(v))
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Real(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(v.to_owned())
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Blob(v)
    }
}

impl From<&[u8]> for Value {
    fn from(v: &[u8]) -> Self {
        Value::Blob(v.to_vec())
    }
}

/// `None` becomes `NULL`; `Some(v)` delegates to `v`'s own conversion.
impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(v: Option<T>) -> Self {
        match v {
            Some(inner) => inner.into(),
            None => Value::Null,
        }
    }
}

#[cfg(feature = "sqlite")]
impl rusqlite::types::ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        use rusqlite::types::{Null, ToSqlOutput};
        Ok(match self {
            Value::Null => ToSqlOutput::from(Null),
            Value::Bool(b) => ToSqlOutput::from(*b),
            Value::Integer(i) => ToSqlOutput::from(*i),
            Value::Real(f) => ToSqlOutput::from(*f),
            Value::Text(s) => ToSqlOutput::from(s.as_str()),
            Value::Blob(b) => ToSqlOutput::from(b.as_slice()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_conversions_widen_to_i64() {
        assert_eq!(Value::from(7_i32), Value::Integer(7));
        assert_eq!(Value::from(7_u8), Value::Integer(7));
        assert_eq!(Value::from(-7_i64), Value::Integer(-7));
    }

    #[test]
    fn option_none_becomes_null() {
        assert_eq!(Value::from(None::<i64>), Value::Null);
        assert_eq!(Value::from(Some(3_i64)), Value::Integer(3));
        assert_eq!(Value::from(Some("x")), Value::Text("x".into()));
    }

    #[test]
    fn text_and_blob_conversions() {
        assert_eq!(Value::from("hi"), Value::Text("hi".into()));
        assert_eq!(Value::from(String::from("hi")), Value::Text("hi".into()));
        assert_eq!(Value::from(&b"ab"[..]), Value::Blob(vec![b'a', b'b']));
    }
}
