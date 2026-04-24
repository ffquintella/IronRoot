//! # ironroot-core
//!
//! Core traits and foundational abstractions for the IronRoot framework.
//!
//! This crate is intentionally lightweight — it has no async runtime or I/O
//! dependencies and can be used in any Rust project regardless of target.
//!
//! ## Key abstractions
//!
//! - [`Entity`] — domain objects that have a stable, unique identity.
//! - [`Service`] — stateless or stateful components that encapsulate business logic.

pub mod entity;
pub mod service;

pub use entity::Entity;
pub use service::Service;
