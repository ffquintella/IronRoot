# IronRoot CLI App Template

A minimal starter for building structured command-line tools with the IronRoot framework.

## What this template demonstrates

- Project layout for an IronRoot CLI application.
- Connection point to `ironroot-cli` for command registration and dispatch.
- Connection point to `ironroot-core` for domain modelling.

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
```

The placeholder binary echoes the provided arguments and prints usage hints.
Once `ironroot-cli` is implemented (Phase 3), replace the `main.rs` body with
a real `CliApp` that registers typed commands.

## Next steps

1. Add `ironroot-cli` to `[dependencies]` in `Cargo.toml`.
2. Create a `CliApp`, register `Command` implementations, and call `.run()`.
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
