# IronRoot Desktop App Template

A minimal starter for building native desktop applications with the IronRoot framework.

## What this template demonstrates

- Project layout for an IronRoot desktop application.
- Connection point to `ironroot-gui` for window and event-loop management.
- Connection point to `ironroot-core` for domain modelling.
- The two-layer test setup [AGENTS.md](AGENTS.md) §4 requires, already wired up and passing.

## Layout

```
src/lib.rs                     everything worth testing — start here
src/main.rs                    one line: print what the library rendered
tests/features/banner.feature  Gherkin scenarios (§4.2)
tests/bdd.rs                   cucumber step definitions and runner
```

Put your code in `src/lib.rs`. A function defined in `main` — or inside a UI callback —
cannot be called from a unit test or from a scenario, so it can only ever count *against*
the 80% coverage gate. The scenarios here run with no window and no display server, which is
the point: keep the shell thin and the logic testable.

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

# Both test layers, then the coverage gate the AGENTS.md checklist enforces
cargo test
cargo llvm-cov --all-features --workspace --fail-under-lines 80
```

The placeholder binary prints a startup message. Once `ironroot-gui` is
implemented (Phase 3), choose a backend and replace `banner()` in `src/lib.rs` with
a real window initialisation — and update the scenarios alongside it, so they keep
describing what the binary actually does.

## Next steps

1. Add `ironroot-gui` to `[dependencies]` with your chosen feature flag.
2. Create a `GuiApp`, configure the window, and call `.run()`.
3. Define your domain types implementing `Entity` from `ironroot-core`.
4. Keep every UI callback a one-liner that delegates to a helper in `src/lib.rs`, and give
   that helper a unit test and a scenario — including its negative case.

## Contributing rules

Before changing anything in this template — or in a project started from it — read
[AGENTS.md](AGENTS.md). It is the single source of truth for how work is done here:
roadmap-driven planning, semantic versioning, changelog upkeep, unit **and** behaviour tests
with line coverage above 80%, secure-coding requirements, and full audit coverage.
[CLAUDE.md](CLAUDE.md) points Claude Code at the same file.

Record every user-visible change in [CHANGELOG.md](CHANGELOG.md) under `## [Unreleased]`, in
the same commit that makes the change.

See [docs/roadmap.md](../../docs/roadmap.md) for the framework roadmap.
