# Changelog

All notable changes to `ironroot-desktop-app` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

Every user-visible change gets an entry under `## [Unreleased]` **in the same commit that
makes the change** — see [AGENTS.md](AGENTS.md) §3.

## [Unreleased]

### Added

- The template now passes its own coverage gate as shipped. A new library target, `src/lib.rs`,
  exposes `banner() -> String` (the text the binary prints) and unit-tests it in `mod tests`;
  `tests/features/banner.feature` and `tests/bdd.rs` add the cucumber layer that
  [AGENTS.md](AGENTS.md) §4.2 requires. `cargo llvm-cov --all-features --workspace
  --fail-under-lines 80` reports 93.75% line coverage instead of exiting 1 on 0.00%, so a fresh
  copy of the template satisfies the §7 checklist before you write a line of your own code.
  The scenarios run without a window, a display server, or an event loop — the working example
  of the AGENTS.md rule that UI callbacks stay thin and logic lives in `src/`.
- First dependencies, both dev-only: `cucumber` 0.23 and `futures` 0.3, the latter purely to
  supply the executor that drives cucumber's async runner from `fn main` in `tests/bdd.rs`.
  `Cargo.toml` gained a `[[test]] name = "bdd"` target with `harness = false` because cucumber
  brings its own runner. `Cargo.lock` is now meaningful and committed: 106 crates, clean under
  `cargo audit`.

### Changed

- `src/main.rs` is a one-line call to `ironroot_desktop_app::banner()`. The printed output is
  byte-for-byte what it was; the text moved to the library target so both test layers can reach
  it. Add new logic to `src/lib.rs`, not to `main` — code that only runs from `main` cannot be
  called from a unit test or a scenario, and counts against the 80% gate with no way to cover it.

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
