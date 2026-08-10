//! Binary entry point for the IronRoot web application template.
//!
//! Deliberately trivial: everything worth testing lives in
//! [`ironroot_web_app`], the library target next door. See `src/lib.rs` for the
//! rationale and for the TODO list that turns this template into a real server.

fn main() {
    println!("{}", ironroot_web_app::banner());
}
