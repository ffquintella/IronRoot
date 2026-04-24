//! # IronRoot Desktop Application Template
//!
//! This template demonstrates how to build a native desktop application using
//! the IronRoot framework. It connects to [`ironroot_gui`] for window and
//! event-loop management, and [`ironroot_core`] for domain modelling.
//!
//! ## Planned UI backends
//!
//! The `ironroot-gui` crate will support two backends via feature flags:
//!
//! - `egui` — immediate-mode GUI, pure Rust, cross-platform.
//! - `tauri` — web-based UI shell (HTML/CSS/JS) with a Rust backend.
//!
//! ## Running
//!
//! ```bash
//! cargo run
//! ```
//!
//! ## TODO
//!
//! - Add `ironroot-gui` dependency once it is published.
//! - Choose a backend feature flag (`egui` or `tauri`).
//! - Implement a minimal window with a "Hello, IronRoot!" label.

fn main() {
    println!("IronRoot Desktop Application");
    println!("----------------------------");
    println!("TODO: Open a native window using ironroot-gui");
    println!();
    println!("Planned backends (via feature flags):");
    println!("  egui  — immediate-mode, pure Rust");
    println!("  tauri — web-based UI shell");
    println!();
    println!("See templates/desktop-app/README.md for the full guide.");
}
