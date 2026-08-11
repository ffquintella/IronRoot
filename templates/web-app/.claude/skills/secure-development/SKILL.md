---
name: secure-development
description: The binding secure-development rules for this project. Use when writing, reviewing, or designing any code that touches authentication, authorization, passwords, hashing, tokens, sessions, crypto, secrets or configuration, SQL and other queries, user input, request bodies, file uploads, HTTP handlers, CLI arguments, IPC commands, UI callbacks, error handling, logging, or the audit trail — and when adding a dependency, reviewing a diff, writing tests for security-sensitive code, or checking whether a change is done. Enforces the 85% overall / 95% security-sensitive line-coverage floors.
---

# Secure development

The rules below are the project's own, not general advice. They restate
[`AGENTS.md`](../../../AGENTS.md) §4–§7 in the form you need while writing code. Where this file
and `AGENTS.md` disagree, `AGENTS.md` wins and the disagreement is a bug — fix it in the same
pull request.

**A change that violates one of these is rejected, not negotiated.** If a rule genuinely cannot
be met, say so explicitly in the pull request with the reason — never skip it silently and never
weaken the gate instead of the code.

## Non-negotiables at a glance

| # | Rule |
|---|---|
| 1 | One authentication/authorization entry point. Never re-check inline. |
| 2 | Authorize by role/group, per application function. Never by hard-coded identity. |
| 3 | No secret in the repository — code, config, tests, fixtures, or git history. |
| 4 | Passwords are one-way hashes (Argon2id/scrypt). Never encrypted, never compared in SQL. |
| 5 | Parameterized queries only. Allow-lists where a parameter cannot bind. |
| 6 | Bound every input, server-side. Escape every output. |
| 7 | Handle every error. No `unwrap`/`expect`/`panic!` on a request, command, or callback path. |
| 8 | Every security-relevant action is audited (INSERT-only, separate store) and logged. |
| 9 | `unsafe` is forbidden unless unavoidable, isolated, `// SAFETY:`-justified, and tested. |
| 10 | 85% line coverage overall; **95% on every security-sensitive file**. |

## 1. Treat these as untrusted input — always

HTTP request bodies, query strings, path segments, headers, cookies, CLI arguments, environment
variables, file paths, stdin, file uploads, IPC/command payloads, UI form fields, and anything
read back from another service. Untrusted means: bound its size, validate its shape server-side,
reject what you do not recognise, and never interpolate it into a query, a path, a shell command,
or a rendered page.

```rust
// Bound the input before you parse it, not after.
const MAX_NAME: usize = 64;

pub fn parse_name(raw: &str) -> Result<Name, ValidationError> {
    if raw.is_empty() || raw.chars().count() > MAX_NAME {
        return Err(ValidationError::Length { max: MAX_NAME });
    }
    if !raw.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(ValidationError::Charset);
    }
    Ok(Name(raw.to_owned()))
}
```

Client-side validation is a convenience, never a control. Re-validate on the server even when
the UI already did.

## 2. Queries

Bind every value. Never build SQL by concatenation or `format!`, not even "just for the table
name in an internal admin tool".

```rust
// Correct — the value is bound, never interpolated.
let user = sqlx::query_as!(
    User,
    "SELECT id, email, role FROM users WHERE email = $1",
    email.as_str(),
)
.fetch_optional(&pool)
.await?;
```

Where a parameter cannot bind — table name, column name, sort direction — map the input through
an allow-list with a safe default. Manual escaping is not an acceptable primary defence.

```rust
fn sort_column(requested: &str) -> &'static str {
    match requested {
        "email" => "email",
        "created_at" => "created_at",
        _ => "id", // safe default; unknown input is not an error path worth leaking
    }
}
```

## 3. Authentication and authorization

- **One** entry point for the whole application. A handler, command, or UI callback asks it; it
  never re-implements the check.
- Authorize by **role/group**, with granularity per application function. Never by a hard-coded
  user id, email, or username.
- Require a **second factor** for sensitive operations: creating or changing credentials,
  changing a password, changing configuration or permissions, exporting data, restoring a backup.
- Apply **progressive lockout** on sign-in — e.g. 3 failures → 1 min, 5 → 15 min, 7 → 1 h — keyed
  primarily on client IP, enforced **before** the password is checked, server-side.
- Return an **identical** response for "unknown user" and "wrong password" — same body, same
  status, and no timing tell (verify against a dummy hash when the user does not exist).

```rust
// Same work, same answer, whether or not the account exists.
let stored = repo.password_hash(&email).await?;
let hash = stored.as_deref().unwrap_or(DUMMY_ARGON2_HASH);
let ok = verify_password(candidate, hash) && stored.is_some();
if !ok {
    audit.record(Event::LoginFailed { email: &email, ip }).await?;
    return Err(AuthError::InvalidCredentials); // one variant for both cases
}
```

## 4. Secrets and sensitive data

- Secrets come from the environment or a secret manager. `.env` is git-ignored; only
  `.env.example` with placeholder values is committed. **A secret that reaches a commit is
  burned** — rotate it, do not just delete the line.
- Passwords and anything that never needs recovering: one-way hash with a modern KDF (Argon2id,
  scrypt). Never encrypt a password. Never compare one in SQL.
- Data that must be reversible is decrypted in server memory only, for the shortest possible
  time, and zeroized after use (`zeroize`).
- TLS on every hop, internal ones included. No plaintext endpoint anywhere.
- Production data never reaches development or staging without masking first.
- **Never log or persist**: a password (even a wrong one), token, key, session cookie, full
  document/ID number, or an unfiltered request body. Derive a `Debug` impl by hand for any type
  holding one, or wrap it so the value cannot print.

```rust
pub struct Secret(String);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(***)") // a `#[derive(Debug)]` here would leak on every `?` log
    }
}
```

## 5. Errors

- Handle every error. Log it with context and a correlation id; return a generic message that
  carries only that id.
- Never render a stack trace, SQL statement, file path, hostname, or component version to a
  caller. Debug mode stays off outside local development.
- No silent swallowing: no bare `let _ =` on a `Result`, no `unwrap()`, `expect()`, or `panic!`
  on a request, command, or callback path. An unhandled panic is a bug, not an error path.
- Define a domain error per module with `thiserror` and map it at the edge.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("account locked")]
    Locked { retry_after: std::time::Duration },
}

// At the edge: log the detail, return the id.
tracing::error!(correlation_id = %id, error = ?err, "request failed");
(StatusCode::INTERNAL_SERVER_ERROR, format!("request failed (ref {id})"))
```

## 6. Audit trail and logging

They are two mechanisms and stay separate. The audit trail is for accountability; the log is for
diagnostics.

Audit **every** security-relevant event: sign-in success **and** failure, sign-out, account
creation or change, credential creation or change, password change, permission change,
configuration change, data export, backup restore, every administrative action. Record at least
timestamp, actor, event, target, source IP, and a structured detail field — with no secret or
sensitive value in the detail.

The trail lives on a **separate database instance** from production data, and the application's
role holds `INSERT` **only**; immutability is enforced by database grants, never by application
discipline. Asynchronous writes are fine; **silent loss is not** — a failure to audit raises an
alert.

Logging: one library, used everywhere, configured centrally. No `println!`/`eprintln!` outside
`main` startup. At least Info/Warn/Error. House format:

```
[dd/mm/yyyy] hh:mm:ss ; event ; details
[29/07/2026] 14:32:05 ; login.failed ; user=jsilva ip=10.2.3.4 attempt=3 lockout=60s
```

Structured JSON logging is fine provided it keeps the same three fields as keys.

## 7. Nothing that bypasses the rules

No arbitrary-SQL endpoint. No admin screen that runs free-form queries. No support backdoor. No
flag that skips authentication outside production. No mutable global state fed by user input —
configuration is loaded from a trusted source and immutable at runtime; inject dependencies
instead of reaching for singletons. Whoever adds a shortcut owns every misuse of it.

Protect every service endpoint — read-only ones included — with TLS plus an access key or token,
and restrict by source IP where the caller is predictable. Expose the minimum data needed. Avoid
heavy database work on unauthenticated surfaces; cache instead.

## 8. Dependencies

Check the support horizon **before** adopting a dependency; discontinued or unmaintained
components are not allowed. Patch-level updates at least quarterly. A **critical** vulnerability
in a dependency outranks every feature request — fix it first and say so.

```bash
cargo audit          # RUSTSEC advisories
cargo deny check     # advisories, bans, licenses, sources
```

An advisory may only be ignored in `deny.toml` with a written justification naming the upstream
blocker and why the code path is unreachable. "Noisy" is not a justification. Never commit a
`Cargo.lock` change you have not reviewed.

## 9. Coverage: 85% overall, 95% security-sensitive

Both floors are gates, not targets.

```bash
# Overall floor — 85% line coverage.
cargo llvm-cov --all-features --workspace --fail-under-lines 85

# Both floors at once: 85% overall plus 95% on every path in `.security-sensitive`.
./scripts/coverage-gate.py

# Browse what is uncovered.
cargo llvm-cov --all-features --workspace --html
```

- **Any file that implements or enforces one of the rules above is security-sensitive** —
  authentication, authorization, session and token handling, password hashing, crypto, input
  validation, output escaping, query construction, secret loading, the audit trail, lockout,
  rate limiting.
- Declare it in [`.security-sensitive`](../../../.security-sensitive) **in the same commit that
  creates it**. An undeclared security module is an unenforced 95%, which is the whole failure
  mode this gate exists to prevent. Reviewers check the manifest against the diff.
- For those files, 95% of lines is the floor and **every error branch is covered**: the rejected
  input, the denied caller, the expired token, the triggered lockout, the failed audit write. A
  security control with an untested negative case is not tested.
- Coverage never goes **down**, even while above the floor. Add the missing test instead of
  lowering the gate.
- Do not chase the number. A test that executes code without asserting on it is worse than no
  test; an uncovered error branch means a missing test, not an excludable line. Any coverage
  exclusion carries a comment saying why the code is untestable.

Both test layers are mandatory (`AGENTS.md` §4): unit/integration tests **and** at least one
Gherkin scenario per feature and per security control — **including the negative case**: access
denied, input rejected, lockout triggered.

```gherkin
Scenario: A caller without the auditor role cannot export data
  Given a signed-in user with the "viewer" role
  When they request a data export
  Then the request is denied with a generic error
  And the denial is recorded in the audit trail
```

## 10. Before you call it done

- [ ] Every new input is bounded and validated server-side; every output escaped.
- [ ] Every query binds its values; every non-bindable part goes through an allow-list.
- [ ] Authentication and authorization go through the single entry point, by role.
- [ ] No secret, credential, or production data entered the repository.
- [ ] Every error is handled and logged with a correlation id; no `unwrap`/`expect`/`panic!` on a
      request, command, or callback path; no bare `let _ =` on a `Result`.
- [ ] Every security-relevant action is audited and logged.
- [ ] New security-sensitive files are listed in `.security-sensitive`.
- [ ] `./scripts/coverage-gate.py` passes — 85% overall, 95% on security-sensitive files — and
      coverage did not drop.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
      `cargo audit`, and `cargo deny check` are clean.
- [ ] `CHANGELOG.md` has the entry (`Security` category if it fixes a vulnerability, with the
      advisory id), and the roadmap item is ticked.

If your organisation's own security standard conflicts with a rule here, the organisation's
standard wins — and the conflict belongs in a pull request against `AGENTS.md`.
