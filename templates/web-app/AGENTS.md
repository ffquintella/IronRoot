# AI Agent Instructions — IronRoot Web App Template

Rules for any contributor — human or AI assistant — working on this template or on a project
started from it. They apply in addition to the framework-wide
[`ai/AGENTS.md`](../../ai/AGENTS.md) and [`ai/INSTRUCTIONS.md`](../../ai/INSTRUCTIONS.md).

Read this file **before** the first edit of a session.

**The numbered sections below are binding.** Section 7 is the checklist to run before
calling any change complete.

## This template

| | |
|---|---|
| Kind | HTTP application (`ironroot-web` + `ironroot-core`) |
| Package | `ironroot-web-app` |
| Roadmap phase | Phase 3 — Templates (Web / CLI / GUI) |
| Standalone | Yes — not a workspace member; build from this directory |

Because this template serves HTTP, §5.3 (injection, output escaping, input limits, error
handling) and §5.1 (authentication, authorization, lockout) apply to every handler you add,
starting with the first one.

## Architecture map

Decide from this table what to read, what to compile, and what to test. Do not rediscover the
layout by scanning the tree.

| Component | Path | Purpose | Depends on | Tests |
|---|---|---|---|---|
| Library | `src/lib.rs` | `banner` — every decision the placeholder binary makes; the router and handlers go here | — | `mod tests` at the bottom of the file |
| Binary | `src/main.rs` | prints the banner; will bind the port once `ironroot-web` is wired in | library | none, by design — keep it that way |
| Scenarios | [`tests/features/banner.feature`](tests/features/banner.feature) + [`tests/bdd.rs`](tests/bdd.rs) | behaviour, including the negative cases (§4.2) | library | themselves |
| Coverage gate | [`scripts/coverage-gate.py`](scripts/coverage-gate.py) + [`.security-sensitive`](.security-sensitive) | 85% overall, 95% security-sensitive (§4.3) | — | — |
| Secure-dev skill | [`.claude/skills/secure-development/SKILL.md`](.claude/skills/secure-development/SKILL.md) | §5 and §6 in working form | — | — |

`ironroot-web-app` is a standalone crate, not a member of the IronRoot workspace: run
every command below **from this directory**, never from the repository root. Dependencies
point one way,
`main.rs` → `lib.rs` → helper modules; nothing points back, so a change inside a helper module
can only affect that module's tests and the scenarios that reach it.

**Do not read, search, or index** `target/`, `Cargo.lock`, or any coverage output. They hold no
source you need and are the most expensive part of the tree to search. Keep searches inside
`src/` and `tests/`, and prefer a symbol search in a known file over a recursive sweep.

---

## Development loop

Wall-clock time from task to validated change is a first-class metric. Compile the smallest
target that proves the change, run the closest test first, and escalate only on evidence.

```text
read AGENTS.md → find the component above → read only it and its direct dependencies
→ make one coherent change → Level 1 → Level 2 → Level 3 once
```

- **Compile the smallest thing that proves the change** — `cargo check` before `cargo build`,
  one test before the suite.
- **Batch edits.** Finish a coherent change, then validate. Never edit → full build → edit.
- **Never `cargo clean`, never delete `target/`.** Reusing the incremental cache is the single
  largest saving available here; a clean build "to be sure" costs minutes and proves nothing
  the incremental one did not.
- **Escalate on evidence, not on habit.** Level 3 runs once, at the end. CI is the
  authoritative full validation — do not reproduce it after every edit.
- **Parallelize independent work**: reading unrelated files, searching separate paths. Do not
  run two cargo commands against the same `target/` at once — they queue on the same lock and
  finish later than they would in sequence.

### Level 1 — fast, run continuously (seconds)

```bash
cargo fmt --all
cargo check --all-targets  # types and borrows; no codegen, no linking
cargo test --lib banner_points  # the closest tests, filtered by name
```

### Level 2 — component, when a coherent change is finished

```bash
cargo clippy --all-targets -- -D warnings
cargo test --lib  # unit tests only
cargo test --test bdd  # scenarios only
```

### Level 3 — project, once, before calling the change done

```bash
cargo test
./scripts/coverage-gate.py  # 85% overall, 95% security-sensitive
```

The gate recompiles with instrumentation, so run it once per change, not per edit. To re-check
both floors without recompiling, keep the export and re-read it:

```bash
cargo llvm-cov --all-features --workspace --json --output-path target/cov.json
./scripts/coverage-gate.py --json target/cov.json
```

### Level 4 — expensive, only when the change warrants it

```bash
cargo audit && cargo deny check  # after any dependency change — see §6.3
cargo llvm-cov --all-features --workspace --html  # browse uncovered lines
cargo build --release
```

Level 4 is warranted when the change touches a dependency, a security control, or performance.
Otherwise leave it to CI.

---

## 1. Follow the roadmap

Work is roadmap-driven. Before starting anything:

1. Read [`docs/roadmap.md`](../../docs/roadmap.md) and find the phase the task belongs to.
2. If the task is **not** on the roadmap, add it there first (as an unchecked item under the
   right phase) and say so in the pull request. Do not silently widen scope.
3. Tick the roadmap checkbox in the **same** commit that lands the work — never ahead of it.
4. Do not start a later phase while an earlier phase has open items that the task depends on.

Roadmap items are the unit of planning; changelog entries are the unit of record. Every
completed roadmap item produces at least one changelog entry.

---

## 2. Semantic versioning

This template follows [Semantic Versioning 2.0.0](https://semver.org/) — `MAJOR.MINOR.PATCH`.

| Change | Bump |
|---|---|
| Removing or renaming a public item; changing a signature, trait bound, or serialized format; tightening validation that rejects previously accepted input | **MAJOR** |
| New public item, new feature flag, new optional config, new endpoint or command | **MINOR** |
| Bug fix, performance work, docs, internal refactor with no public surface change | **PATCH** |
| Dependency bump | **PATCH**, unless it changes this crate's own public surface or MSRV — then **MINOR** |

Rules:

- **Pre-1.0 (`0.y.z`) is not an excuse.** While the version is `0.y.z`, treat `y` as MAJOR and
  `z` as MINOR/PATCH, and still document every break.
- **No silent breaking changes.** A breaking change must land with a `## [Unreleased]` entry
  under `### Removed` or `### Changed`, and a migration note.
- **Deprecate before removing.** Mark the item `#[deprecated(since = "…", note = "use … instead")]`
  in one release; remove it no earlier than the next MAJOR.
- **Raising the MSRV is at least a MINOR bump** and must be stated in the changelog.
- **Every released version is built from the version-control tree and gets a tag** (`vX.Y.Z`).
  Never ship a build that does not correspond to a tagged commit.
- Bump `version` in `Cargo.toml` and move the `## [Unreleased]` block to a dated
  `## [X.Y.Z] - YYYY-MM-DD` heading in the same commit as the tag.

---

## 3. Keep the changelog updated

The project keeps `CHANGELOG.md` in [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
format. **Every** user-visible change updates it — in the same commit, not afterwards.

```markdown
# Changelog

All notable changes to this project are documented in this file.
The format is based on Keep a Changelog and this project adheres to Semantic Versioning.

## [Unreleased]

### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security

## [0.1.0] - YYYY-MM-DD

### Added
- Initial template scaffold.
```

Rules:

- Add the entry under `## [Unreleased]` in the correct category. Only `Security` is used for
  vulnerability fixes — name the advisory ID (`RUSTSEC-…`, `CVE-…`) when there is one.
- Write for the person **consuming** the change, not for the reviewer: what changed and what
  they must do about it. Not `refactor handler`, but
  `HTTP handlers now return 422 instead of 400 for validation errors`.
- Internal-only refactors with no observable effect may be omitted; when in doubt, include them.
- A change-of-version document is mandatory: the changelog entry must make clear **what changed
  and where** (files, modules, endpoints), so a reviewer can reconstruct the change from it.
- Never rewrite a released section. Corrections go in a new entry.

---

## 4. Tests: unit *and* behaviour, coverage above 85%

Two layers are required. A feature is not done with only one of them.

### 4.1 Unit / integration tests

- `#[test]` (and `#[tokio::test]`) functions in `src/` (`mod tests`) and `tests/*.rs`.
- Cover the happy path, every error branch, and the boundaries — empty input, maximum length,
  zero, overflow, unauthorized caller.
- Tests are deterministic: no wall-clock, no network, no shared global state, no ordering
  dependence between tests. Inject a `Clock`, a repository, an HTTP client.
- Every fixed bug gains a regression test that fails without the fix.

### 4.2 Behaviour (BDD) tests

- Gherkin `.feature` files under `tests/features/`, executed by
  [cucumber-rs](https://crates.io/crates/cucumber), with step definitions in `tests/bdd.rs`.
- Write the **behaviour**, not the implementation:

  ```gherkin
  Feature: Invoice totals
    Scenario: VAT is applied to taxable line items
      Given an invoice with a 100.00 EUR taxable item
      When the totals are calculated
      Then the grand total should be 121.00 EUR
  ```

- Keep steps thin: parse arguments, call one helper from `src/`, assert. No business logic
  inside a step definition.
- Every user-facing feature and every security control (authentication, authorization,
  lockout, input limits) gets at least one scenario, including the **negative** case —
  access denied, input rejected, lockout triggered.

If you cannot exercise a helper from a BDD scenario without starting a server or a database,
the helper is doing too much — split it.

### 4.3 Coverage gates: 85% overall, 95% security-sensitive

Two floors. Both are gates, not targets, and both are enforced by one command.

| Scope | Floor |
|---|---|
| Every line of the project | **85%** |
| Every file listed in [`.security-sensitive`](.security-sensitive) | **95%** |

```bash
cargo install cargo-llvm-cov                                     # once
./scripts/coverage-gate.py                                       # both floors
cargo llvm-cov --all-features --workspace --fail-under-lines 85  # the overall floor alone
cargo llvm-cov --all-features --workspace --html                 # browse uncovered lines
```

- **Line coverage must stay above 85% overall.** A change that pushes it below the threshold is
  not mergeable; add the missing tests instead of lowering the gate.
- **Security-sensitive code must stay above 95%**, and every one of its error branches must be
  covered: the rejected input, the denied caller, the expired token, the triggered lockout, the
  failed audit write. A security control whose negative case is untested is not tested.
- A file is security-sensitive when it implements or enforces a control from §5 or §6 —
  authentication, authorization, session or token handling, password hashing, crypto, input
  validation, output escaping, query construction, secret loading, lockout, rate limiting, or the
  audit trail. **Declare it in `.security-sensitive` in the same commit that creates it.** An
  undeclared security module is an unenforced 95%, which is the failure mode the manifest exists
  to prevent; reviewers check it against the diff.
- Coverage never goes **down** in a pull request, even while above a floor.
- Do not chase the number with assertion-free tests. An uncovered error branch means a missing
  test; a test that executes code without asserting on it is worse than no test.
- `#[cfg(not(tarpaulin_include))]`-style exclusions and `#[coverage(off)]` need a comment
  justifying why the code is untestable. They are never how a file reaches 95%.

Run before every push:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
./scripts/coverage-gate.py
```

---

## 5. Secure development practices

These are requirements, not suggestions. Reviewers reject changes that violate them.

The [`secure-development` skill](.claude/skills/secure-development/SKILL.md) is the working form
of this section and §6: the same rules, with the Rust patterns that satisfy them and the review
checklist to run before calling a change done. Claude Code loads it automatically; other
assistants should be pointed at it. Keep the two in step — where they disagree, this file wins
and the disagreement is a bug to fix in the same pull request.

### 5.1 Authentication and authorization

- **One** authentication and authorization entry point for the whole application. Never
  re-implement a check inline in a handler, a command, or a UI callback.
- Authorize by **role/group**, never by hard-coded user identity, with granularity per
  application function.
- Require a second factor for sensitive operations: creating or changing credentials, changing
  a password, changing configuration, exporting data, restoring a backup, changing permissions.
- Apply progressive lockout on login — e.g. 3 failures → 1 min, 5 → 15 min, 7 → 1 h — keyed
  primarily on client IP, enforced **before** the password is checked, server-side. Return an
  identical response for "unknown user" and "wrong password".

### 5.2 Data and secrets

- **No secret ever enters the repository** — not in code, not in committed config, not in
  tests, not in fixtures, not in the git history. Secrets come from the environment or a
  secret manager; `.env` is git-ignored and only `.env.example` (with placeholder values) is
  committed.
- Passwords and anything else that never needs recovering are stored as a **one-way hash**
  with a modern KDF (Argon2id / scrypt). Never encrypt a password, never compare one in SQL.
- Data that must be reversible is decrypted **in server memory only**, for the shortest
  possible time, and zeroized after use (`zeroize`).
- Sensitive data travels **encrypted only**: TLS on every hop, including internal ones. No
  plaintext endpoint anywhere.
- Production data never reaches development or staging without passing through a masking
  step first.
- Never log or persist a password (even a wrong one), token, key, session cookie, full
  document number, or unfiltered request body.

### 5.3 Code

- **Parameterized queries only.** Never interpolate input — or any part of a URL — into SQL.
  Where parameters cannot bind (table name, sort column, sort direction), use an allow-list
  with a safe default. Manual escaping is not an acceptable primary defence.
- **Escape on output.** No user-controlled HTML or JavaScript reaches a rendered page. Rely on
  the template engine's auto-escaping; sanitize with an allow-list where markup is genuinely
  allowed. Validate server-side — client-side validation is a convenience, not a control.
- **Bound every input**, including URLs, query strings, request bodies, and file uploads.
  Enforce the limit server-side and reject with a handled error.
- **Handle every error.** Log it with context and a correlation id; return a generic message
  carrying only that id. Never render a stack trace, SQL statement, file path, hostname, or
  component version. Never swallow an error silently — no bare `let _ =` on a `Result`, no
  `unwrap()`/`expect()`/`panic!` in request or command paths. Debug mode stays off outside
  local development.
- **No mutable global state fed by user input.** Configuration is loaded from a trusted
  source and treated as immutable at runtime. Inject dependencies instead of reaching for
  singletons.
- **Protect every service endpoint** with TLS plus an access key or token — read-only
  endpoints included — and restrict by source IP where the caller is predictable. Expose the
  minimum data needed.
- **Never build a diagnostic shortcut**: no arbitrary-SQL endpoint, no admin screen that runs
  free-form queries, no support backdoor, no flag that skips authentication outside
  production. Whoever adds one owns every misuse of it.
- **`unsafe` is forbidden** unless unavoidable. If unavoidable: isolate it in its own module,
  justify the invariant in a `// SAFETY:` comment, and cover it with a test.
- Avoid heavy database work on unauthenticated surfaces; cache instead.

### 5.4 Dependencies

- Discontinued or unmaintained components are not allowed. Check the support horizon **before**
  adopting a dependency.
- Patch-level updates at least quarterly; key frameworks reviewed at least every six months.
- A **critical** vulnerability in a dependency outranks every feature request. Fix it first,
  and say so instead of continuing with the feature work.

---

## 6. Full audit coverage

"Audit" means two separate obligations. Both are mandatory.

### 6.1 Audit trail — who did what

An audit trail is not a log. Keep the two mechanisms separate.

| | Audit trail | Log |
|---|---|---|
| Purpose | Accountability | Diagnostics |
| Store | Separate instance from production data | Application log sink |
| Mutability | INSERT-only, enforced by the database | Rotated freely |

Requirements:

- Audit **every** security-relevant event. At minimum: sign-in (success **and** failure),
  sign-out, account creation, account change, credential creation or change, password change,
  permission change, configuration change, data export, backup restore, and every
  administrative action.
- Write the trail to a **different database instance** from production data.
- The application's database role holds `INSERT` **only** — no `UPDATE`, no `DELETE`. The
  immutability is enforced by database grants and constraints, never by application discipline.
- Optimize the table for cheap inserts: minimal indexes, no heavy triggers on the write path.
- Record at least: timestamp, actor, event, target, source IP, and a structured detail field.
  Never put a secret or sensitive value in the detail field.
- Asynchronous writes are fine; **silent loss is not**. A failure to audit raises an alert.
- Document the audit mechanism — events covered, schema, retention — in `docs/`. Retention is
  agreed with the security team; do not invent a period.

### 6.2 Logging

- **One** logging library, used everywhere. No `println!`/`eprintln!` outside `main` startup.
- At least three levels: **Info** (routine), **Warn** (needs attention), **Error** (problems).
- Every error is logged. Authentication events and changes to important data are always logged.
- Configure the formatter once, centrally — do not assemble log strings at each call site. The
  house format is:

  ```
  [dd/mm/yyyy] hh:mm:ss ; event ; details
  ```

  For example: `[29/07/2026] 14:32:05 ; login.failed ; user=jsilva ip=10.2.3.4 attempt=3 lockout=60s`

  Structured (JSON) logging is acceptable provided it keeps the same three fields as keys.

### 6.3 Supply-chain and code audit

Run in CI on every pull request, and locally before a release:

```bash
cargo audit                    # RUSTSEC advisories
cargo deny check               # advisories, bans, licenses, sources
cargo clippy --all-targets -- -D warnings
```

- An advisory may only be added to a `deny.toml` ignore list with a written justification
  naming the upstream blocker and why the code path is unreachable. "Noisy" is not a
  justification.
- A **critical or high** finding blocks the release and blocks new feature work until fixed.
- Never commit `Cargo.lock` changes you have not reviewed.

---

## 7. Definition of done

A change is complete only when **all** of these hold:

- [ ] It maps to a roadmap item in [`docs/roadmap.md`](../../docs/roadmap.md), ticked in the same commit.
- [ ] `CHANGELOG.md` has an entry under `## [Unreleased]` in the right category.
- [ ] The version bump matches the semver rules in §2 (or the change is unreleased).
- [ ] Unit/integration tests cover the happy path, the error branches, and the boundaries.
- [ ] At least one BDD scenario covers the behaviour, including its negative case.
- [ ] `./scripts/coverage-gate.py` passes — 85% of lines overall, 95% on every security-sensitive
      file — and coverage did not drop.
- [ ] Every new file that implements or enforces a security control is listed in
      [`.security-sensitive`](.security-sensitive).
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` pass.
- [ ] `cargo audit` and `cargo deny check` are clean.
- [ ] Every security-relevant action the change introduces is audited (§6.1) and logged (§6.2).
- [ ] No secret, credential, or production data was added to the repository.
- [ ] Every new public item has a `///` doc comment; every new module has a `//!` comment.

---

## 8. Where the rules come from

- [`docs/roadmap.md`](../../docs/roadmap.md) — what to build, and in what order.
- [`ai/AGENTS.md`](../../ai/AGENTS.md) — framework-wide agent rules.
- [`ai/INSTRUCTIONS.md`](../../ai/INSTRUCTIONS.md) — naming, layout, and extension conventions.
- [`docs/architecture.md`](../../docs/architecture.md) — layered architecture.
- [`.claude/skills/secure-development/SKILL.md`](.claude/skills/secure-development/SKILL.md) —
  §5 and §6 in working form, for you and for any AI assistant.
- [`.security-sensitive`](.security-sensitive) — which paths the 95% coverage floor applies to.

If a rule here conflicts with your organisation's own security standard, the organisation's
standard wins — and the conflict belongs in a pull request against this file.

Update this file whenever a convention changes.
