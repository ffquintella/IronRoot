# IronRoot CLI App Template

A minimal starter for building structured command-line tools with the IronRoot framework.

## What this template demonstrates

- Project layout for an IronRoot CLI application.
- Connection point to `ironroot-cli` for command registration and dispatch.
- Connection point to `ironroot-core` for domain modelling.
- The two-layer test setup [AGENTS.md](AGENTS.md) §4 requires, already wired up and passing.

## Layout

```
src/lib.rs                       everything worth testing — start here
src/main.rs                      collect the arguments, print what the library rendered
tests/features/dispatch.feature  Gherkin scenarios (§4.2)
tests/bdd.rs                     cucumber step definitions and runner
```

Put your code in `src/lib.rs`. A function defined in `main` cannot be called from a unit test
or from a scenario, so it can only ever count *against* the 80% coverage gate. Note that
`render` receives the arguments as a parameter instead of reading `std::env::args` itself —
that is what lets a scenario drive it.

## IronRoot crates used

| Crate | Role |
|---|---|
| `ironroot-core` | Domain traits (`Entity`, `Service`) |
| `ironroot-cli` *(planned)* | Command registration and argument dispatch |

## Getting started

```bash
# From the template directory
cargo build
cargo run
cargo run -- greet Alice

# Both test layers, then the coverage gate the AGENTS.md checklist enforces
cargo test
cargo llvm-cov --all-features --workspace --fail-under-lines 80
```

The placeholder binary echoes the provided arguments and prints usage hints.
Once `ironroot-cli` is implemented (Phase 3), replace `render()` in `src/lib.rs` with
a real `CliApp` that registers typed commands — and update the scenarios alongside it,
so they keep describing what the binary actually does.

## Next steps

1. Add `ironroot-cli` to `[dependencies]` in `Cargo.toml`.
2. Create a `CliApp`, register `Command` implementations, and call `.run()`.
3. Define your domain types implementing `Entity` from `ironroot-core`.
4. Give every command a unit test and a scenario, including its negative case — an
   unknown command, a missing argument, an over-long input.

## Contributing rules

Before changing anything in this template — or in a project started from it — read
[AGENTS.md](AGENTS.md). It is the single source of truth for how work is done here:
roadmap-driven planning, semantic versioning, changelog upkeep, unit **and** behaviour tests
with line coverage above 80%, secure-coding requirements, and full audit coverage.
[CLAUDE.md](CLAUDE.md) points Claude Code at the same file.

Record every user-visible change in [CHANGELOG.md](CHANGELOG.md) under `## [Unreleased]`, in
the same commit that makes the change.

See [docs/roadmap.md](../../docs/roadmap.md) for the framework roadmap.
