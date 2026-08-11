# CLAUDE.md — IronRoot Web App Template

**All project instructions live in [AGENTS.md](AGENTS.md). Read that file first; it is the
single source of truth.** This file exists only so that Claude Code picks the rules up
automatically — it deliberately duplicates nothing.

> @AGENTS.md

The secure-development rules in AGENTS.md §5–§6 also ship as a skill you load automatically:
[`.claude/skills/secure-development/SKILL.md`](.claude/skills/secure-development/SKILL.md). Use it
whenever a change touches authentication, authorization, secrets, queries, untrusted input, error
handling, logging, the audit trail, or a dependency — and before calling any change done.

## Quick reminders (the full rules are in AGENTS.md)

| Topic | Rule | Section |
|---|---|---|
| Roadmap | Every change maps to an item in [`docs/roadmap.md`](../../docs/roadmap.md); tick it in the same commit | §1 |
| Versioning | Semantic Versioning; no silent breaking changes; tag every release | §2 |
| Changelog | Update `CHANGELOG.md` under `## [Unreleased]` in the same commit | §3 |
| Tests | Unit/integration **and** BDD scenarios — both, every feature | §4 |
| Coverage | `./scripts/coverage-gate.py` must pass — **85%** of lines overall, **95%** on every file in [`.security-sensitive`](.security-sensitive) — and coverage must not drop | §4.3 |
| Secure code | Parameterized queries, output escaping, bounded input, handled errors, no secrets in the repo, no `unsafe` | §5 |
| Audit | Every security-relevant action audited to an INSERT-only store on a separate instance; `cargo audit` + `cargo deny check` clean | §6 |
| Done | Work through the checklist before saying a change is finished | §7 |

Every HTTP handler you add is a security boundary: authenticate and authorize through the
single entry point, bound the input, escape the output, handle the error, audit the action.

## Before you report a change as complete

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
./scripts/coverage-gate.py    # 85% overall, 95% security-sensitive
cargo audit && cargo deny check
```

Do not report work as done until these pass and the AGENTS.md §7 checklist is satisfied.
