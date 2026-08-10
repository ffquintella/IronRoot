//! # IronRoot CLI Application Template
//!
//! This template demonstrates how to build a structured CLI tool using the
//! IronRoot framework. It connects to [`ironroot_cli`] for command registration
//! and dispatch, and [`ironroot_core`] for domain modelling.
//!
//! ## Why the logic lives here and not in `main`
//!
//! Everything the binary prints lives in this library target so both test layers
//! required by [`AGENTS.md`](../AGENTS.md) §4 can reach it:
//!
//! - unit tests in the `tests` module at the bottom of this file (§4.1),
//! - cucumber scenarios in `tests/features/dispatch.feature`, executed by
//!   `tests/bdd.rs` (§4.2).
//!
//! `src/main.rs` does one thing — collect the arguments and hand them to
//! [`render`]. Keep it that way. Code that lives in `main` cannot be called from
//! either layer, so it counts against the 80% line coverage gate in §4.3 without
//! any way to cover it. It is also why [`render`] takes its arguments as a
//! parameter instead of reading [`std::env::args`] itself: a helper that reaches
//! for process state cannot be driven from a scenario.
//!
//! ## Running
//!
//! ```bash
//! cargo run -- --help
//! cargo run -- greet Alice
//! ```
//!
//! ## TODO
//!
//! - Add `ironroot-cli` dependency once it is published.
//! - Register at least one real `Command` implementation.
//! - Wire argument parsing (clap or argh) through the `ironroot-cli` adapter.

/// Renders the title block printed before anything else.
#[must_use]
pub fn banner() -> String {
    ["IronRoot CLI Application", "------------------------"].join("\n")
}

/// Renders everything the binary prints for `args`.
///
/// `args` holds the arguments *after* the program name, exactly as
/// `std::env::args().skip(1)` yields them. Passing them in rather than reading
/// the environment is what makes this function testable from both layers.
///
/// With no arguments the caller gets usage text; with arguments it gets a
/// placeholder acknowledgement of what real dispatch would do. Replace the
/// second branch when `ironroot-cli` lands — and remember §5 applies from the
/// first real command: arguments are untrusted input, so bound their length and
/// reject what you do not recognise instead of passing them on.
#[must_use]
pub fn render(args: &[String]) -> String {
    let mut out = banner();
    out.push('\n');

    if args.is_empty() {
        out.push_str(&usage());
    } else {
        out.push_str(&dispatch_notice(args));
    }

    out
}

/// Renders the usage text shown when the binary is invoked with no arguments.
fn usage() -> String {
    [
        "Usage: ironroot-cli-app <command> [args...]",
        "",
        "Available commands (placeholder):",
        "  greet <name>   Print a greeting",
        "",
        "See templates/cli-app/README.md for the full guide.",
    ]
    .join("\n")
}

/// Renders the placeholder acknowledgement shown when arguments were supplied.
fn dispatch_notice(args: &[String]) -> String {
    format!(
        "Arguments received: {args:?}\n\nTODO: Dispatch to the matching Command via ironroot-cli"
    )
}

#[cfg(test)]
mod tests {
    use super::{banner, render};

    /// Builds an owned argument vector the way `main` would.
    fn args(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn banner_underlines_the_title() {
        let rendered = banner();
        let mut lines = rendered.lines();
        let title = lines.next().expect("banner starts with a title line");
        let rule = lines.next().expect("banner has a rule under the title");

        assert_eq!(rule.chars().count(), title.chars().count());
        assert!(rule.chars().all(|c| c == '-'));
        assert!(lines.next().is_none(), "the banner is only the title block");
    }

    #[test]
    fn every_invocation_starts_with_the_banner() {
        for argv in [args(&[]), args(&["greet", "Alice"])] {
            assert!(render(&argv).starts_with(&banner()));
        }
    }

    #[test]
    fn no_arguments_prints_usage() {
        let rendered = render(&args(&[]));

        assert!(rendered.contains("Usage: ironroot-cli-app <command> [args...]"));
        assert!(rendered.contains("greet <name>"));
        assert!(rendered.contains("templates/cli-app/README.md"));
        assert!(!rendered.contains("Arguments received"));
    }

    #[test]
    fn arguments_are_echoed_back() {
        let rendered = render(&args(&["greet", "Alice"]));

        assert!(rendered.contains(r#"Arguments received: ["greet", "Alice"]"#));
        assert!(rendered.contains("TODO: Dispatch to the matching Command via ironroot-cli"));
        assert!(!rendered.contains("Usage:"));
    }

    #[test]
    fn a_single_empty_argument_is_still_an_argument() {
        // The boundary between the two branches is "was anything passed", not
        // "was anything non-empty" — `--` and `""` reach dispatch, not usage.
        let rendered = render(&args(&[""]));

        assert!(rendered.contains(r#"Arguments received: [""]"#));
        assert!(!rendered.contains("Usage:"));
    }

    #[test]
    fn rendering_is_deterministic_and_untrailed() {
        let argv = args(&["greet", "Alice"]);

        assert_eq!(render(&argv), render(&argv));
        assert!(!render(&argv).ends_with('\n'));
        assert!(!render(&args(&[])).ends_with('\n'));
    }
}
