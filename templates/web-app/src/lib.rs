//! # IronRoot Web Application Template
//!
//! This template demonstrates how to build an HTTP application using the
//! IronRoot framework. It connects to [`ironroot_web`] for routing and
//! middleware, and [`ironroot_core`] for domain modelling.
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
//! `src/main.rs` is a one-liner, and should stay one. Code that lives in `main`
//! cannot be called from either layer, so it counts against the 80% line
//! coverage gate in §4.3 without any way to cover it.
//!
//! ## Running
//!
//! ```bash
//! cargo run
//! # The server will listen on http://127.0.0.1:8080
//! ```
//!
//! ## TODO
//!
//! - Add `ironroot-web` dependency once it is published.
//! - Register at least one route that returns a JSON response.
//! - Add middleware for request logging.

/// Renders the startup banner the binary prints.
///
/// The wording is deliberately inert. This template does not start a server
/// yet, so the banner describes what is still missing rather than implying that
/// anything is listening. Replace the body when you replace `main`, and keep the
/// scenarios in `tests/features/banner.feature` honest about what the new
/// version claims.
#[must_use]
pub fn banner() -> String {
    [
        "IronRoot Web Application",
        "------------------------",
        "TODO: Start HTTP server on 127.0.0.1:8080",
        "      - Wire up ironroot-web Router",
        "      - Register routes",
        "      - Apply middleware",
        "",
        "See templates/web-app/README.md for the full guide.",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::banner;

    #[test]
    fn banner_names_the_application() {
        assert!(banner().starts_with("IronRoot Web Application"));
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
    fn banner_points_at_the_readme() {
        assert!(banner().contains("templates/web-app/README.md"));
    }

    #[test]
    fn banner_does_not_claim_a_running_server() {
        let rendered = banner().to_lowercase();

        assert!(!rendered.contains("listening on"));
        assert!(!rendered.contains("server started"));
    }

    #[test]
    fn banner_is_a_single_block_without_a_trailing_newline() {
        let rendered = banner();

        assert!(!rendered.ends_with('\n'));
        assert!(rendered.lines().count() > 1);
    }
}
