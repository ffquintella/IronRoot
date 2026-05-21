# `ironroot-dal-hiqlite`

Hiqlite integration for the IronRoot DAL family.

[Hiqlite](https://crates.io/crates/hiqlite) is a Raft-backed embedded SQLite
that can run as a highly-available cluster — embedded inside your binary,
no separate database process required.

## Why is this a standalone crate?

Hiqlite links the `sqlite3` native library via `rusqlite`, and so does
`sqlx-sqlite`. Cargo refuses to build a dependency graph that activates
both — **even when only one feature is enabled at a time**, because the
resolver pessimistically considers all reachable `links` consumers.

Keeping hiqlite in its own crate (excluded from the main IronRoot workspace
via `workspace.exclude`) avoids the conflict for projects that don't need
hiqlite. The crate lives at `crates/dal-hiqlite/` and is built independently:

```bash
cd crates/dal-hiqlite
cargo build
```

For the same reason this crate intentionally does **not** depend on
[`ironroot-dal`](dal.md); the small `DalError` / `ExecResult` shapes are
mirrored locally and are API-compatible with their counterparts there.

## What's in it

- `HiqlitePool` — wraps an `Arc<hiqlite::Client>` and exposes `execute` /
  `execute_params` matching the `ExecResult` shape.
- `pool.client()` — drop down to the underlying `hiqlite::Client` for full
  read APIs (`query_raw`, `query_map`, transactions, batch, cache).
- Re-export `pub use hiqlite;` so downstream code only needs one dep.

## Cargo

```toml
[dependencies]
ironroot-dal-hiqlite = { path = "../IronRoot/crates/dal-hiqlite" }
```

## Example

```rust,ignore
use ironroot_dal_hiqlite::{HiqlitePool, hiqlite};
use std::sync::Arc;

// Construct the hiqlite client however your application requires
// (`hiqlite::start_node_with_cache`, single-node embedded mode, etc.).
let client: hiqlite::Client = /* ... */;
let pool = HiqlitePool::new(Arc::new(client));

// Schema management and writes go through `execute`:
pool.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)").await?;

// Reads use hiqlite's typed query APIs directly:
let users: Vec<User> = pool.client()
    .query_as("SELECT id, name FROM users", hiqlite::params!())
    .await?;
```

## Error type

```rust
pub enum DalError {
    Database(String),
}
```

`HiqlitePool::execute` returns `Result<ExecResult, DalError>` where
`ExecResult` is the same shape as `ironroot_dal::ExecResult` —
`rows_affected: u64`, `last_insert_id: Option<i64>` (always `None` for
hiqlite as its high-level `execute` does not surface it).

## See also

- [`ironroot-dal`](dal.md) — the sqlx-based DAL for everything else.
- [Hiqlite docs](https://docs.rs/hiqlite) — Raft setup, leadership,
  consensus tuning, dashboard.
