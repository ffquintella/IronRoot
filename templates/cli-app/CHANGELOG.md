# Changelog

All notable changes to `ironroot-cli-app` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

Every user-visible change gets an entry under `## [Unreleased]` **in the same commit that
makes the change** — see [AGENTS.md](AGENTS.md) §3.

## [Unreleased]

### Added

- A `secure-development` skill now ships with the template, at
  `.claude/skills/secure-development/SKILL.md`. Claude Code loads it automatically in this
  template and in any project started from it, and it carries [AGENTS.md](AGENTS.md) §5–§6 in
  working form: what counts as untrusted input, parameterized queries and allow-lists for the
  parts that cannot bind, the single authentication and authorization entry point, progressive
  lockout with identical responses for unknown-user and wrong-password, secret and password
  handling, error handling that leaks nothing to the caller, the INSERT-only audit trail on a
  separate instance, dependency hygiene — each with the Rust pattern that satisfies it, and the
  checklist to run before calling a change done.
- `scripts/coverage-gate.py` enforces both coverage floors in one command: **85%** of lines
  overall and **95%** on every path declared in the new `.security-sensitive` manifest. It prints
  a per-file table, names the files that miss a floor, and exits 1 — or 2 when the gate itself
  cannot run, such as a missing `cargo-llvm-cov` or an unreadable report. `--json PATH` re-checks
  an existing `cargo llvm-cov` export without re-running the suite. The script exists because
  `cargo llvm-cov --fail-under-lines` enforces one project-wide number and cannot express a
  per-file floor; nothing in the build depends on it.
- `.security-sensitive` declares which paths the 95% floor applies to — one `fnmatch` pattern per
  line, shipped with the conventional module names (`src/auth*`, `src/crypto*`, `src/session*`,
  `src/token*`, `src/audit*`, …), none of which match a file yet. Add a path there in the same
  commit that creates a file implementing or enforcing a security control: an undeclared security
  module is an unenforced 95%, so reviewers check the manifest against the diff.
- The template now passes its own coverage gate as shipped. A new library target, `src/lib.rs`,
  exposes `banner() -> String` and `render(&[String]) -> String` (the text the binary prints for
  a given argument list) and unit-tests both branches in `mod tests`;
  `tests/features/dispatch.feature` and `tests/bdd.rs` add the cucumber layer that
  [AGENTS.md](AGENTS.md) §4.2 requires, including the negative case — no arguments prints usage
  and never echoes an argument list. `cargo llvm-cov --all-features --workspace
  --fail-under-lines 85` reports 94.52% line coverage instead of exiting 1 on 0.00%, so a fresh
  copy of the template satisfies the §7 checklist before you write a line of your own code.
- First dependencies, both dev-only: `cucumber` 0.23 and `futures` 0.3, the latter purely to
  supply the executor that drives cucumber's async runner from `fn main` in `tests/bdd.rs`.
  `Cargo.toml` gained a `[[test]] name = "bdd"` target with `harness = false` because cucumber
  brings its own runner. `Cargo.lock` is now meaningful and committed: 106 crates, clean under
  `cargo audit`.

### Changed

- The line-coverage floor is **85%**, up from 80%, and security-sensitive files carry a separate
  **95%** floor under which every error branch must be covered — the rejected input, the denied
  caller, the expired token, the triggered lockout, the failed audit write. [AGENTS.md](AGENTS.md)
  §4.3 is rewritten around the two floors and the §7 checklist now runs
  `./scripts/coverage-gate.py` instead of a bare `cargo llvm-cov --fail-under-lines 80`;
  `README.md` and `CLAUDE.md` follow, and both point at the new `secure-development` skill. As
  shipped the template reports 94.52% overall and declares no security-sensitive file yet, so
  a fresh copy passes both gates before you write a line of your own code.
- `src/main.rs` now only collects `std::env::args().skip(1)` and passes the result to
  `ironroot_cli_app::render`. Both printed outputs — usage and the argument echo — are
  byte-for-byte what they were; the text and the branch between them moved to the library target
  so both test layers can reach them. `render` takes the arguments as a parameter rather than
  reading `std::env::args` itself, because a helper that reaches for process state cannot be
  driven from a scenario. Add new logic to `src/lib.rs`, not to `main` — code that only runs
  from `main` cannot be called from a unit test or a scenario, and counts against the 85% gate
  with no way to cover it.

### Deprecated

### Removed

### Fixed

- `cargo build` and `cargo run` now work from this directory. `Cargo.toml` gained an empty
  `[workspace]` table so Cargo treats the template as its own workspace root; without it,
  Cargo walked up to the IronRoot repository root and failed with "current package believes
  it's in a workspace when it's not". The template is still deliberately not a member of the
  IronRoot workspace, which now also lists it under `workspace.exclude`.

### Security

## [0.1.0] - 2026-04-24

### Added

- Initial template scaffold: `Cargo.toml`, placeholder `src/main.rs`, and `README.md`.
