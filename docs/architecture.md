# IronRoot — Architecture

## Overview

IronRoot is structured around a layered architecture that separates concerns into distinct crates. Each layer depends only on layers below it, keeping coupling low and testability high.

```
┌─────────────────────────────────────────┐
│            Application Layer            │
│  (templates: web-app / cli-app / gui)   │
├─────────────────────────────────────────┤
│            Interface Layer              │
│      crates/web  crates/cli  crates/gui │
├─────────────────────────────────────────┤
│          Infrastructure Layer           │
│          (future: db, cache, …)         │
├─────────────────────────────────────────┤
│            Domain / Core Layer          │
│              crates/core                │
├─────────────────────────────────────────┤
│            Cross-cutting               │
│             crates/macros               │
└─────────────────────────────────────────┘
```

---

## Crate Responsibilities

### `ironroot-core`

- Defines foundational traits (`Entity`, `Service`) used across all other crates.
- Intentionally lightweight — no async runtime, no I/O.
- OOP helpers (the external [`classes`](https://crates.io/crates/classes) crate and
  [`inherit-methods-macro`](https://crates.io/crates/inherit-methods-macro)) may be
  added as *optional* `[dependencies]` by consumers of this crate when class-like
  patterns are preferred, but they are **not bundled with IronRoot**. Any usage must
  be clearly documented.

### `ironroot-macros`

- Procedural macro crate (`proc-macro = true`).
- Provides derive macros (`#[derive(Entity)]`) that implement `ironroot-core` traits automatically.
- All generated code must be readable and deterministic — no hidden side-effects.

### `ironroot-web`

- *(Planned)* HTTP server integration layer.
- Will expose traits for `Router`, `Middleware`, `Handler`, and `HttpContext`.
- Designed to be backend-agnostic (Axum, Actix-web, etc.) via trait adapters.

### `ironroot-cli`

- Provides `CliApp`, `Command`, and `CommandContext` abstractions.
- Thin wrapper intended to unify argument parsing libraries behind a common interface.

### `ironroot-gui`

- *(Planned)* Desktop UI integration.
- Will support Tauri (web-based) and/or egui (immediate-mode) via feature flags.

### `ironroot-log`

- Default logging facade built on `tracing`.
- Size-based file rotation (10 MB default, 5 retained files).
- Optional syslog export (`syslog` feature) — Unix socket or remote UDP.
- Exposes a `LogConfig` builder and a one-call `init_default(app_name)` shortcut.

### `ironroot-dal`

- Data Access Layer built on `sqlx`.
- Routes database access between backends at runtime: **SQLite** and **MySQL**
  are enabled by default; **PostgreSQL** is available behind the `postgres`
  feature flag.
- Backend is selected from the connection-URL scheme (`sqlite:`, `mysql:`,
  `postgres:`) — no recompilation required to switch.
- Exposes a `Pool` enum for raw query execution and a `Repository` trait for
  uniform CRUD-style access over domain types.

### `ironroot-dal-hiqlite` *(standalone, not a workspace member)*

- Optional integration with [hiqlite](https://crates.io/crates/hiqlite), a
  Raft-backed embedded SQLite for highly-available deployments.
- Exposed as a separate `HiqlitePool` wrapper rather than a `Pool` variant
  because hiqlite's `Row<'_>` carries a lifetime that doesn't compose with
  the lifetime-free `Row` used by the rest of `ironroot-dal`.
- Lives in `crates/dal-hiqlite/` but is **excluded** from the workspace
  because hiqlite (via `rusqlite`) and `sqlx-sqlite` both link the
  `sqlite3` native library and Cargo refuses to resolve a graph with both.
  Build it standalone with `cargo build` from `crates/dal-hiqlite/`.

### `ironroot-auth`

- Authentication primitives: `User`, `Credentials`, `NewUser`,
  `PasswordHasher`, `AuthService`.
- **Argon2id** for password hashing — already post-quantum-secure as a
  memory-hard symmetric KDF.
- Optional `pq-seal` feature: wraps stored Argon2id PHC strings in an
  authenticated envelope using **ML-KEM-768** (NIST FIPS 203) for key
  encapsulation and **ChaCha20-Poly1305** for AEAD, giving a named PQ layer
  at rest.
- Optional `dal` feature: `SqlUserRepository` backed by `ironroot-dal`.

---

## Macro System

IronRoot macros follow these rules:

1. **Derive macros only** — avoid attribute macros unless necessary for ergonomics.
2. **Generate idiomatic code** — the expansion should look like handwritten Rust.
3. **Document expansion** — every macro must include a doc comment showing example input and its generated output.
4. **No `unsafe` in generated code** unless it cannot be avoided and is clearly justified.

---

## OOP Support Strategy

IronRoot takes a **traits-first** approach to object-oriented patterns:

1. **Traits** are the primary abstraction mechanism.
2. **Generics and associated types** encode polymorphism without virtual dispatch overhead.
3. **Dynamic dispatch (`dyn Trait`)** is used only at crate boundaries or when heterogeneous collections are required.
4. **External helper crates** — the [`classes`](https://crates.io/crates/classes) crate
   and [`inherit-methods-macro`](https://crates.io/crates/inherit-methods-macro) provide
   ergonomic shortcuts for class-like patterns and method inheritance. They are **not**
   bundled with IronRoot; add them to your own `[dependencies]` when needed. Any usage
   must be justified and documented in the module where they appear.

---

## Dependency Rules

| Crate | May depend on |
|---|---|
| `ironroot-macros` | `proc-macro2`, `quote`, `syn` only |
| `ironroot-core` | `ironroot-macros` (optional) |
| `ironroot-web` | `ironroot-core` |
| `ironroot-cli` | `ironroot-core` |
| `ironroot-gui` | `ironroot-core` |
| `ironroot-log` | `tracing`, `tracing-subscriber`, `tracing-appender`, `file-rotate`, `syslog` (optional) |
| `ironroot-dal` | `sqlx` (sqlite + mysql by default, postgres optional), `async-trait`, `thiserror`, `tracing` |
| `ironroot-dal-hiqlite` | `hiqlite`, `thiserror`, `tracing` (standalone — not a workspace member) |
| `ironroot-auth` | `argon2`, `password-hash`, `rand`, `thiserror`, `async-trait`; optional `ironroot-dal`, `ml-kem`, `chacha20poly1305`, `sha2` |
| Templates | Any of the above crates |

No crate in `crates/` may depend on a `templates/` project.
