//! # ironroot-web
//!
//! Web layer abstractions for the IronRoot framework.
//!
//! This crate provides the building blocks for HTTP-based applications built on
//! top of [`ironroot_core`]. It is intentionally backend-agnostic: concrete
//! implementations for specific HTTP frameworks (Axum, Actix-web, etc.) will
//! live in separate adapter crates.
//!
//! ## Planned modules
//!
//! - `router` — route registration and dispatch
//! - `middleware` — composable request/response transformations
//! - `handler` — async request handlers
//! - `context` — per-request state container
//!
//! # TODO
//!
//! Implement the above modules in Phase 3 of the roadmap.

pub mod router;

/// Re-export core so downstream crates only need one dependency.
pub use ironroot_core;
