//! # IronRoot CLI Application Template
//!
//! This template demonstrates how to build a structured CLI tool using the
//! IronRoot framework. It connects to [`ironroot_cli`] for command registration
//! and dispatch, and [`ironroot_core`] for domain modelling.
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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    println!("IronRoot CLI Application");
    println!("------------------------");

    if args.len() > 1 {
        println!("Arguments received: {:?}", &args[1..]);
        println!();
        println!("TODO: Dispatch to the matching Command via ironroot-cli");
    } else {
        println!("Usage: ironroot-cli-app <command> [args...]");
        println!();
        println!("Available commands (placeholder):");
        println!("  greet <name>   Print a greeting");
        println!();
        println!("See templates/cli-app/README.md for the full guide.");
    }
}
