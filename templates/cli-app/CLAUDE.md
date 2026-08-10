# CLAUDE.md — IronRoot CLI App Template

**All project instructions live in [AGENTS.md](AGENTS.md). Read that file first; it is the
single source of truth.** This file exists only so that Claude Code picks the rules up
automatically — it deliberately duplicates nothing.

> @AGENTS.md

## Quick reminders (the full rules are in AGENTS.md)

| Topic | Rule | Section |
|---|---|---|
| Roadmap | Every change maps to an item in [`docs/roadmap.md`](../../docs/roadmap.md); tick it in the same commit | §1 |
| Versioning | Semantic Versioning; no silent breaking changes; tag every release | §2 |
| Changelog | Update `CHANGELOG.md` under `## [Unreleased]` in the same commit | §3 |
| Tests | Unit/integration **and** BDD scenarios — both, every feature | §4 |
| Coverage | `cargo llvm-cov --all-features --workspace --fail-under-lines 80` must pass, and coverage must not drop | §4.3 |
| Secure code | Parameterized queries, output escaping, bounded input, handled errors, no secrets in the repo, no `unsafe` | §5 |
| Audit | Every security-relevant action audited to an INSERT-only store on a separate instance; `cargo audit` + `cargo deny check` clean | §6 |
| Done | Work through the checklist before saying a change is finished | §7 |

Arguments, environment variables, file paths, and stdin are untrusted input. Bound them,
validate them, and never interpolate them into a query or a shell command. An unhandled panic
is a bug, not an error path.

## Before you report a change as complete

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo llvm-cov --all-features --workspace --fail-under-lines 80
cargo audit && cargo deny check
```

Do not report work as done until these pass and the AGENTS.md §7 checklist is satisfied.
