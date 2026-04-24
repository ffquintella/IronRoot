//! # IronRoot Web Application Template
//!
//! This template demonstrates how to build an HTTP application using the
//! IronRoot framework. It connects to [`ironroot_web`] for routing and
//! middleware, and [`ironroot_core`] for domain modelling.
//!
//! ## Running
//!
//! ```bash
//! cargo run
//! # The server listens on http://127.0.0.1:8080
//! ```
//!
//! ## TODO
//!
//! - Add `ironroot-web` dependency once it is published.
//! - Register at least one route that returns a JSON response.
//! - Add middleware for request logging.

fn main() {
    println!("IronRoot Web Application");
    println!("------------------------");
    println!("TODO: Start HTTP server on 127.0.0.1:8080");
    println!("      - Wire up ironroot-web Router");
    println!("      - Register routes");
    println!("      - Apply middleware");
    println!();
    println!("See templates/web-app/README.md for the full guide.");
}
