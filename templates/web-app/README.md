# IronRoot Web App Template

A minimal starter for building HTTP applications with the IronRoot framework.

## What this template demonstrates

- Project layout for an IronRoot web application.
- Connection point to `ironroot-web` for routing and middleware.
- Connection point to `ironroot-core` for domain modelling.

## IronRoot crates used

| Crate | Role |
|---|---|
| `ironroot-core` | Domain traits (`Entity`, `Service`) |
| `ironroot-web` *(planned)* | HTTP routing and middleware |

## Getting started

```bash
# From the template directory
cargo build
cargo run
```

The placeholder binary prints a startup message. Once `ironroot-web` is
implemented (Phase 3 of the roadmap), replace the `main.rs` body with real
HTTP server initialisation.

## Next steps

1. Add `ironroot-web` to `[dependencies]` in `Cargo.toml`.
2. Create a `Router`, register handlers, and call `.serve("127.0.0.1:8080")`.
3. Define your domain types implementing `Entity` from `ironroot-core`.
4. Add middleware for logging, authentication, etc.

See [docs/roadmap.md](../../docs/roadmap.md) for the framework roadmap.
