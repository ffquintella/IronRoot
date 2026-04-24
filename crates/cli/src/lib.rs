//! # ironroot-cli
//!
//! CLI abstraction layer for the IronRoot framework.
//!
//! This crate provides traits and types for building structured command-line
//! applications on top of [`ironroot_core`]. It is designed to be
//! argument-parser-agnostic; concrete integrations (clap, argh, etc.) will be
//! provided via optional feature flags in the future.
//!
//! ## Modules
//!
//! - [`app`] — top-level CLI application container
//! - [`command`] — individual command abstraction

pub mod app;
pub mod command;

pub use app::CliApp;
pub use command::Command;

/// Re-export core so downstream crates only need one dependency.
pub use ironroot_core;
