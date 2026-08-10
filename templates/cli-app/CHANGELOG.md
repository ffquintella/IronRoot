# Changelog

All notable changes to `ironroot-cli-app` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

Every user-visible change gets an entry under `## [Unreleased]` **in the same commit that
makes the change** — see [AGENTS.md](AGENTS.md) §3.

## [Unreleased]

### Added

### Changed

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
