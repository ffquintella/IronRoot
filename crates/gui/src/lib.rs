//! # ironroot-gui
//!
//! Desktop UI placeholder for the IronRoot framework.
//!
//! This crate will integrate one or more desktop UI libraries — currently
//! under evaluation:
//!
//! - **[egui](https://github.com/emilk/egui)** — immediate-mode GUI, pure Rust.
//! - **[Tauri](https://tauri.app/)** — web-based UI shell with a Rust backend.
//!
//! The choice of backend will be controlled via Cargo feature flags so that
//! projects can opt in to the toolkit that best fits their needs.
//!
//! # TODO
//!
//! - Evaluate egui vs Tauri for Phase 3.
//! - Define `Window`, `Application`, and `EventLoop` abstractions.
//! - Add feature flags `feature = ["egui"]` and `feature = ["tauri"]`.

pub mod app;

pub use app::GuiApp;

/// Re-export core so downstream crates only need one dependency.
pub use ironroot_core;
