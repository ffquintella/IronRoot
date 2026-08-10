# IronRoot Web App Template

A minimal starter for building HTTP applications with the IronRoot framework.

## What this template demonstrates

- Project layout for an IronRoot web application.
- Connection point to `ironroot-web` for routing and middleware.
- Connection point to `ironroot-core` for domain modelling.
- The two-layer test setup [AGENTS.md](AGENTS.md) §4 requires, already wired up and passing.

## Layout

```
src/lib.rs                     everything worth testing — start here
src/main.rs                    one line: print what the library rendered
tests/features/banner.feature  Gherkin scenarios (§4.2)
tests/bdd.rs                   cucumber step definitions and runner
```

Put your code in `src/lib.rs`. A function defined in `main` cannot be called from a unit test
or from a scenario, so it can only ever count *against* the 80% coverage gate.

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

# Both test layers, then the coverage gate the AGENTS.md checklist enforces
cargo test
cargo llvm-cov --all-features --workspace --fail-under-lines 80
```

The placeholder binary prints a startup message. Once `ironroot-web` is
implemented (Phase 3 of the roadmap), replace `banner()` in `src/lib.rs` with real
HTTP server initialisation — and update the scenarios alongside it, so they keep
describing what the binary actually does.

## Next steps

1. Add `ironroot-web` to `[dependencies]` in `Cargo.toml`.
2. Create a `Router`, register handlers, and call `.serve("127.0.0.1:8080")`.
3. Define your domain types implementing `Entity` from `ironroot-core`.
4. Add middleware for logging, authentication, etc.
5. Give every handler a unit test and a scenario, including its negative case — an
   unauthorized caller, an over-long body, a rejected field.

## Contributing rules

Before changing anything in this template — or in a project started from it — read
[AGENTS.md](AGENTS.md). It is the single source of truth for how work is done here:
roadmap-driven planning, semantic versioning, changelog upkeep, unit **and** behaviour tests
with line coverage above 80%, secure-coding requirements, and full audit coverage.
[CLAUDE.md](CLAUDE.md) points Claude Code at the same file.

Record every user-visible change in [CHANGELOG.md](CHANGELOG.md) under `## [Unreleased]`, in
the same commit that makes the change.

See [docs/roadmap.md](../../docs/roadmap.md) for the framework roadmap.
