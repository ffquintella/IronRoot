# ironroot-dal-hiqlite

Hiqlite (Raft-backed embedded SQLite) integration for [`ironroot-dal`].

## Why is this a standalone crate?

Both [`hiqlite`](https://crates.io/crates/hiqlite) (via `rusqlite`) and
`sqlx-sqlite` declare `links = "sqlite3"` on the same native library. Cargo
refuses to build any dependency graph that activates both at once.

To keep that conflict from breaking the main IronRoot workspace, this crate
is **excluded** from the workspace `members` list (see the `exclude` entry
in the top-level `Cargo.toml`). It builds independently:

```bash
cd crates/dal-hiqlite
cargo build
```

It depends on `ironroot-dal` with `default-features = false, features =
["mysql"]` so the sqlx-sqlite driver never enters the graph for this crate.

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
