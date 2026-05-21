# IronRoot

> A modular, full-stack Rust framework inspired by ASP.NET Zero — bringing structured, layered architecture and object-oriented patterns into idiomatic Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org/)

---

## Overview

IronRoot provides a foundation for building scalable, layered Rust applications with expressive abstractions and tooling support. It is designed for developers who want the productivity and structure of enterprise frameworks — without sacrificing the safety and performance that Rust delivers.

IronRoot is:

- **Modular** — pick only the crates you need.
- **Macro-driven** — reduce boilerplate with proc-macros.
- **OOP-friendly** — optional class-like abstractions via helper crates.
- **Full-stack** — templates for web, CLI, and desktop (GUI) applications.
- **AI-ready** — first-class support for AI agents and MCP (Model Context Protocol) integration.

---

## Key Features

| Feature | Description |
|---|---|
| Modular crate architecture | Each concern lives in its own crate under `crates/` |
| Macro-driven abstractions | Derive macros like `#[derive(Entity)]` reduce boilerplate |
| OOP-like support | Traits first; optional helpers (`classes`, `inherit-methods-macro`) for richer patterns |
| Multi-interface templates | Starter projects for web, CLI, and desktop applications |
| AI + MCP readiness | Built-in AI agent conventions and MCP server scaffolding |

---

## Project Structure

```
ironroot/
├── Cargo.toml              # Workspace root
├── README.md
├── .gitignore
├── docs/
│   ├── architecture.md     # Layered architecture overview
│   └── roadmap.md          # Development roadmap
├── ai/
│   ├── AGENTS.md           # AI agent roles and rules
│   └── INSTRUCTIONS.md     # Extension and contribution guidelines
├── mcp/
│   ├── servers/
│   │   └── example_server/ # Minimal MCP server binary
│   └── README.md
├── crates/
│   ├── core/               # Base traits: Entity, Service
│   ├── macros/             # Procedural macros (#[derive(Entity)], …)
│   ├── web/                # Web layer abstractions (future)
│   ├── cli/                # CLI abstraction layer
│   └── gui/                # Desktop UI placeholder (Tauri/egui)
└── templates/
    ├── web-app/            # Standalone web application template
    ├── cli-app/            # Standalone CLI application template
    └── desktop-app/        # Standalone desktop application template
```

---

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, 1.75+)
- `cargo` (included with Rust)

### Build the workspace

```bash
git clone https://github.com/ffquintella/IronRoot.git
cd IronRoot
cargo build
```

### Bootstrap a new project (interactive)

```bash
cargo run -p ironroot-new
```

`ironroot-new` asks for the project kind (CLI tool, web app, client/server),
the GUI toolkit (Tauri or egui — only for client/server), the frontend
framework (React or Angular — only when a web UI is involved), and the
database, then writes out a ready-to-build project with `Cargo.toml`,
`Makefile`, basic tests, and an `AGENTS.md`.

### Use a template

```bash
# Or copy a template manually to start a new project
cp -r templates/cli-app my-cli-app
cd my-cli-app
cargo build && cargo run
```

---

## Crate Overview

### `ironroot-core`

Provides the foundational traits (`Entity`, `Service`) that all domain objects and services implement. This is the only mandatory dependency for IronRoot-based projects.

### `ironroot-macros`

Procedural macros (`#[derive(Entity)]`, etc.) that reduce boilerplate when implementing core traits.

### `ironroot-web`

*(Planned)* HTTP layer integration — routing, middleware, request/response abstractions.

### `ironroot-cli`

Provides abstractions for building command-line applications with structured argument parsing and command dispatch.

### `ironroot-gui`

*(Planned)* Desktop UI integration layer. Future support for Tauri and/or egui.

### `ironroot-log`

Default logging facade built on `tracing`. Provides 10 MB size-based file
rotation (configurable), retains 5 historical files, and offers optional
syslog export via the `syslog` feature flag. One-call setup:

```rust
let _guard = ironroot_log::init_default("my-app")?;
tracing::info!("ready");
```

### `ironroot-dal`

Data Access Layer built on `sqlx`. Routes database access between SQLite and
MySQL by default (PostgreSQL behind the `postgres` feature) — the backend is
selected from the connection-URL scheme at runtime.

```rust
use ironroot_dal::Pool;

let pool = Pool::connect("sqlite::memory:").await?;
// or: Pool::connect("mysql://user:pw@host:3306/app").await?;
pool.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").await?;
```

### `ironroot-dal-hiqlite` *(standalone)*

Optional integration with [hiqlite](https://crates.io/crates/hiqlite) for
Raft-backed embedded SQLite. Lives at `crates/dal-hiqlite/` and is built
independently from the main workspace — see its [README](crates/dal-hiqlite/README.md)
for why.

### `ironroot-auth`

Authentication primitives — `User` entities, `Credentials`, `AuthService` —
with **Argon2id** password hashing (post-quantum-secure as a memory-hard
symmetric KDF). The optional `pq-seal` feature additionally wraps stored
hashes in an authenticated envelope using **ML-KEM-768** (NIST FIPS 203,
ex-Kyber) + ChaCha20-Poly1305 for a named PQ-cryptography layer at rest.
The optional `dal` feature ships a `SqlUserRepository` backed by `ironroot-dal`.

```rust
use ironroot_auth::{AuthService, NewUser, Credentials, SqlUserRepository, PqSealer};
use ironroot_dal::Pool;
use std::sync::Arc;

let pool = Pool::connect("sqlite::memory:").await?;
let repo = SqlUserRepository::new(pool);
repo.ensure_schema().await?;

let svc = AuthService::new(repo)
    .with_pq_sealer(Arc::new(PqSealer::generate())); // optional PQ layer

let _user = svc.register(NewUser {
    username: "ada".into(),
    email: "ada@example.org".into(),
    password: "correct horse battery staple".into(),
}).await?;

let _verified = svc.verify(&Credentials {
    username: "ada".into(),
    password: "correct horse battery staple".into(),
}).await?;
```

---

## Documentation

The `docs/` folder is a [docsify](https://docsify.js.org) site — plain
Markdown rendered live in the browser, no build step. Serve it locally:

```bash
make docs              # → http://localhost:3000
# or directly:
./docs/serve_docs.sh   # Unix
docs\serve_docs.bat    # Windows
```

Both scripts prefer `docsify-cli` (`npm i -g docsify-cli`) and fall back to
Python's `http.server` if Node isn't available.

Direct file links (also browsable on GitHub):

- [Architecture](docs/architecture.md) — layered design, crate responsibilities, macro system
- [Roadmap](docs/roadmap.md) — phased development plan
- Per-crate docs under [`docs/crates/`](docs/crates/) (one page per crate)
- [AI Agents](ai/AGENTS.md) — conventions for AI-assisted development
- [Extension Guide](ai/INSTRUCTIONS.md) — how to add crates, macros, and templates
- [MCP Integration](mcp/README.md) — Model Context Protocol server scaffolding

---

## Roadmap

See [docs/roadmap.md](docs/roadmap.md) for the full phased plan.

| Phase | Focus |
|---|---|
| 1 | Core abstractions (`core` crate) |
| 2 | Macro system (`macros` crate) |
| 3 | Templates (web / CLI / GUI) |
| 4 | MCP + AI agents |
| 5 | Ecosystem expansion |

---

## Contributing

Contributions are welcome! Please read the [extension guide](ai/INSTRUCTIONS.md) and follow the conventions described in [AGENTS.md](ai/AGENTS.md) before opening a pull request.

---

## License

[MIT](LICENSE)
