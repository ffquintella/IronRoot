# ironroot-dal-hiqlite

Hiqlite (Raft-backed embedded SQLite) integration for [`ironroot-dal`].

## Relationship to `ironroot-dal`

`ironroot-dal`'s SQLite backend is built on `rusqlite` — the same crate
hiqlite uses internally — so both can coexist in one Cargo workspace
without the historical `links = "sqlite3"` clash that `sqlx-sqlite` used
to cause. This crate is a regular workspace member and builds with the
rest of IronRoot via `cargo build --workspace`.

`ironroot-dal-hiqlite` does not depend on `ironroot-dal` directly so
callers that only need hiqlite can skip sqlx (and its MySQL / PostgreSQL
transitive deps). The small `DalError` / `ExecResult` shapes are
duplicated intentionally for that reason.

## Usage

```rust,ignore
use ironroot_dal_hiqlite::{HiqlitePool, hiqlite};
use std::sync::Arc;

// You construct the hiqlite client however your application requires
// (`hiqlite::start_node_with_cache`, single-node embedded mode, etc.).
let client: hiqlite::Client = /* ... */;
let pool = HiqlitePool::new(Arc::new(client));

// Schema management and writes go through `execute`:
pool.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)").await?;

// Reads use hiqlite's typed query APIs directly:
let client = pool.client();
let users: Vec<User> = client.query_as("SELECT id, name FROM users", hiqlite::params!()).await?;
```
