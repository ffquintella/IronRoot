//! Behaviour tests for `ironroot-web-app` — AGENTS.md §4.2.
//!
//! Gherkin sources live in `tests/features/`; this file only binds their steps
//! to helpers from `src/`. Keep steps thin — parse the argument, call one
//! helper, assert. Business logic in a step definition is logic the unit tests
//! and the coverage gate never see.
//!
//! This target sets `harness = false` in `Cargo.toml` because cucumber brings
//! its own runner; `fn main` at the bottom is what `cargo test` executes.

use cucumber::{World, then, when};

/// State carried between the steps of a single scenario.
#[derive(Debug, Default, World)]
struct BannerWorld {
    /// Text produced by the most recent render step.
    rendered: Option<String>,
}

impl BannerWorld {
    /// Returns the text rendered earlier in the scenario.
    ///
    /// # Panics
    ///
    /// Panics if no render step ran first — a scenario wiring mistake, which
    /// should fail loudly rather than assert against an empty string.
    fn rendered(&self) -> &str {
        self.rendered
            .as_deref()
            .expect("a `When` step must render the banner before a `Then` step inspects it")
    }
}

#[when("the startup banner is rendered")]
fn render_banner(world: &mut BannerWorld) {
    world.rendered = Some(ironroot_web_app::banner());
}

#[then(expr = "the banner should start with {string}")]
fn banner_starts_with(world: &mut BannerWorld, expected: String) {
    let rendered = world.rendered();
    assert!(
        rendered.starts_with(&expected),
        "expected the banner to start with {expected:?}, got:\n{rendered}"
    );
}

#[then(expr = "the banner should contain {string}")]
fn banner_contains(world: &mut BannerWorld, expected: String) {
    let rendered = world.rendered();
    assert!(
        rendered.contains(&expected),
        "expected the banner to contain {expected:?}, got:\n{rendered}"
    );
}

#[then(expr = "the banner should not contain {string}")]
fn banner_does_not_contain(world: &mut BannerWorld, forbidden: String) {
    let rendered = world.rendered().to_lowercase();
    assert!(
        !rendered.contains(&forbidden.to_lowercase()),
        "expected the banner not to contain {forbidden:?}, got:\n{rendered}"
    );
}

fn main() {
    futures::executor::block_on(BannerWorld::run("tests/features"));
}
