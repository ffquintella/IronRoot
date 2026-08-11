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
scripts/coverage-gate.py       the 85% / 95% coverage gates (§4.3)
.security-sensitive            paths the 95% coverage floor applies to (§4.3)
.claude/skills/                the secure-development skill, loaded automatically (§5–§6)
```

Put your code in `src/lib.rs`. A function defined in `main` cannot be called from a unit test
or from a scenario, so it can only ever count *against* the 85% coverage gate.

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

# Both test layers, then the coverage gates the AGENTS.md checklist enforces
cargo test
./scripts/coverage-gate.py    # 85% of lines overall, 95% on security-sensitive paths
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

## Secure development and coverage

The rules that matter most while you write code ship with the template instead of living in a
wiki:

- [`.claude/skills/secure-development/SKILL.md`](.claude/skills/secure-development/SKILL.md) —
  [AGENTS.md](AGENTS.md) §5–§6 in working form: untrusted input, parameterized queries, one
  authentication entry point, secrets, error handling, the audit trail, dependency hygiene, and
  the checklist to run before calling a change done. Claude Code loads it automatically; point
  other assistants at it.
- [`scripts/coverage-gate.py`](scripts/coverage-gate.py) — enforces both floors from §4.3: **85%**
  line coverage overall and **95%** on every path listed in
  [`.security-sensitive`](.security-sensitive). Declare a path there in the same commit that adds
  a file implementing or enforcing a security control.

## Contributing rules

Before changing anything in this template — or in a project started from it — read
[AGENTS.md](AGENTS.md). It is the single source of truth for how work is done here:
roadmap-driven planning, semantic versioning, changelog upkeep, unit **and** behaviour tests
with line coverage above 85% — 95% on security-sensitive paths — secure-coding requirements,
and full audit coverage. [CLAUDE.md](CLAUDE.md) points Claude Code at the same file, and the
[`secure-development` skill](.claude/skills/secure-development/SKILL.md) carries §5–§6 in the form
an AI assistant loads on its own.

Record every user-visible change in [CHANGELOG.md](CHANGELOG.md) under `## [Unreleased]`, in
the same commit that makes the change.

See [docs/roadmap.md](../../docs/roadmap.md) for the framework roadmap.
