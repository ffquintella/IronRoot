# `ironroot-dal`

Data Access Layer for IronRoot — wraps [`sqlx`](https://docs.rs/sqlx) with a
runtime-routed connection pool.

## What's in it

- `Pool` — an enum that dispatches to the right `sqlx` backend at runtime.
- `Backend` — `Sqlite` / `MySql` / `Postgres`, parsed from the connection URL.
- `Row` — backend-agnostic row wrapper with `try_get_*` accessors.
- `Repository` — generic async CRUD trait for domain entities.
- `ExecResult`, `DalError` — common return / error types.

For Raft-backed embedded SQLite, see the standalone
[`ironroot-dal-hiqlite`](dal-hiqlite.md) crate.

## Feature flags

| Feature | Default | Backend |
|---|---|---|
| `sqlite` | ✅ | SQLite via `sqlx-sqlite` |
| `mysql`  | ✅ | MySQL / MariaDB via `sqlx-mysql` |
| `postgres` | — | PostgreSQL via `sqlx-postgres` |

```toml
[dependencies]
# default (SQLite + MySQL)
ironroot-dal = "0.2"

# add Postgres
ironroot-dal = { version = "0.2", features = ["postgres"] }

# MySQL-only build (smaller binary)
ironroot-dal = { version = "0.2", default-features = false, features = ["mysql"] }
```

## Backend selection

The backend is chosen from the URL scheme — no recompilation:

| Scheme | Backend |
|---|---|
| `sqlite:` | SQLite |
| `mysql:` / `mariadb:` | MySQL |
| `postgres:` / `postgresql:` | PostgreSQL |

## Quick start

```rust
use ironroot_dal::{Pool, DalError};

# async fn run() -> Result<(), DalError> {
let pool = Pool::connect("sqlite::memory:").await?;
// or: Pool::connect("mysql://user:pw@localhost:3306/app").await?;

pool.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").await?;
pool.execute("INSERT INTO users (id, name) VALUES (1, 'Ada')").await?;

let rows = pool.fetch_all("SELECT id, name FROM users").await?;
assert_eq!(rows.len(), 1);
assert_eq!(rows[0].try_get_string("name")?, "Ada");
# Ok(()) }
```

## Repositories

```rust
use async_trait::async_trait;
use ironroot_dal::{DalError, Pool, Repository, Row};

pub struct User { pub id: i64, pub name: String }

pub struct UserRepo<'a> { pub pool: &'a Pool }

#[async_trait]
impl<'a> Repository for UserRepo<'a> {
    type Entity = User;
    type Id = i64;

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, DalError> {
        let sql = format!("SELECT id, name FROM users WHERE id = {id}");
        Ok(self.pool.fetch_optional(&sql).await?
           .map(|r| Ok::<_, DalError>(User {
               id: r.try_get_i64("id")?, name: r.try_get_string("name")?,
           })).transpose()?)
    }

    async fn list(&self) -> Result<Vec<User>, DalError> {
        let rows = self.pool.fetch_all("SELECT id, name FROM users").await?;
        rows.into_iter()
            .map(|r| Ok(User { id: r.try_get_i64("id")?, name: r.try_get_string("name")? }))
            .collect()
    }

    async fn delete(&self, id: i64) -> Result<(), DalError> {
        let sql = format!("DELETE FROM users WHERE id = {id}");
        self.pool.execute(&sql).await?;
        Ok(())
    }
}
```

## Row accessors

The `Row` wrapper exposes typed accessors that work across backends:

```rust
let row = pool.fetch_optional("SELECT 1 AS n, 'ada' AS name").await?.unwrap();
let n:    i64    = row.try_get_i64("n")?;
let name: String = row.try_get_string("name")?;
```

For values outside the basic set (`i64`, `String`, `bool`, `f64`), drop down
to sqlx directly via the per-backend modules.

## Error type

```rust
pub enum DalError {
    UnsupportedUrl(String),
    Database(String),
    Decode { column: String, message: String },
    NotFound,
}
```

## See also

- [`ironroot-auth`](auth.md) — uses `ironroot-dal` for user persistence.
- [`ironroot-dal-hiqlite`](dal-hiqlite.md) — Raft-backed embedded SQLite.
