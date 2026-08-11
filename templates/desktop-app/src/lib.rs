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
//! ## Why the logic lives here and not in `main`
//!
//! Everything the binary prints lives in this library target so both test layers
//! required by [`AGENTS.md`](../AGENTS.md) §4 can reach it:
//!
//! - unit tests in the `tests` module at the bottom of this file (§4.1),
//! - a cucumber scenario in `tests/features/banner.feature`, executed by
//!   `tests/bdd.rs` (§4.2).
//!
//! `src/main.rs` is a one-liner, and should stay one. This is the same rule
//! AGENTS.md states for UI callbacks: keep the shell thin and the logic in
//! `src/`, so it can be unit- and BDD-tested without opening a window. Code that
//! only runs from `main` — or only from an event handler — counts against the
//! 85% line coverage gate in §4.3 with no way to cover it.
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

/// Renders the startup banner the binary prints.
///
/// The wording is deliberately inert. This template does not open a window or
/// select a backend yet, so the banner lists what is *planned* rather than
/// implying that anything was started. Replace the body when you replace `main`,
/// and keep the scenarios in `tests/features/banner.feature` honest about what
/// the new version claims.
#[must_use]
pub fn banner() -> String {
    [
        "IronRoot Desktop Application",
        "----------------------------",
        "TODO: Open a native window using ironroot-gui",
        "",
        "Planned backends (via feature flags):",
        "  egui  — immediate-mode, pure Rust",
        "  tauri — web-based UI shell",
        "",
        "See templates/desktop-app/README.md for the full guide.",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::banner;

    #[test]
    fn banner_names_the_application() {
        assert!(banner().starts_with("IronRoot Desktop Application"));
    }

    #[test]
    fn banner_underlines_the_title() {
        let rendered = banner();
        let mut lines = rendered.lines();
        let title = lines.next().expect("banner starts with a title line");
        let rule = lines.next().expect("banner has a rule under the title");

        assert_eq!(rule.chars().count(), title.chars().count());
        assert!(rule.chars().all(|c| c == '-'));
    }

    #[test]
    fn banner_lists_both_planned_backends() {
        let rendered = banner();

        assert!(rendered.contains("egui"));
        assert!(rendered.contains("tauri"));
    }

    #[test]
    fn banner_does_not_claim_a_window_or_a_chosen_backend() {
        let rendered = banner().to_lowercase();

        assert!(!rendered.contains("window opened"));
        assert!(!rendered.contains("backend selected"));
        assert!(rendered.contains("planned backends"));
    }

    #[test]
    fn banner_points_at_the_readme() {
        assert!(banner().contains("templates/desktop-app/README.md"));
    }

    #[test]
    fn banner_is_a_single_block_without_a_trailing_newline() {
        let rendered = banner();

        assert!(!rendered.ends_with('\n'));
        assert!(rendered.lines().count() > 1);
    }
}
