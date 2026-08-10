# IronRoot Desktop App Template

A minimal starter for building native desktop applications with the IronRoot framework.

## What this template demonstrates

- Project layout for an IronRoot desktop application.
- Connection point to `ironroot-gui` for window and event-loop management.
- Connection point to `ironroot-core` for domain modelling.

## IronRoot crates used

| Crate | Role |
|---|---|
| `ironroot-core` | Domain traits (`Entity`, `Service`) |
| `ironroot-gui` *(planned)* | Window management and event loop |

## Planned UI backends

`ironroot-gui` will offer two backends via Cargo feature flags:

| Feature | Backend | Notes |
|---|---|---|
| `egui` | [egui](https://github.com/emilk/egui) | Immediate-mode, pure Rust |
| `tauri` | [Tauri](https://tauri.app/) | Web UI shell with Rust backend |

## Getting started

```bash
# From the template directory
cargo build
cargo run
```

The placeholder binary prints a startup message. Once `ironroot-gui` is
implemented (Phase 3), choose a backend and replace the `main.rs` body with
a real window initialisation.

## Next steps

1. Add `ironroot-gui` to `[dependencies]` with your chosen feature flag.
2. Create a `GuiApp`, configure the window, and call `.run()`.
3. Define your domain types implementing `Entity` from `ironroot-core`.

## Contributing rules

Before changing anything in this template — or in a project started from it — read
[AGENTS.md](AGENTS.md). It is the single source of truth for how work is done here:
roadmap-driven planning, semantic versioning, changelog upkeep, unit **and** behaviour tests
with line coverage above 80%, secure-coding requirements, and full audit coverage.
[CLAUDE.md](CLAUDE.md) points Claude Code at the same file.

Record every user-visible change in [CHANGELOG.md](CHANGELOG.md) under `## [Unreleased]`, in
the same commit that makes the change.

See [docs/roadmap.md](../../docs/roadmap.md) for the framework roadmap.
