//! Pure Rust primitives for deterministic, app-scoped synthetic identifiers.
//!
//! This crate does not install Android hooks, write system properties, or alter
//! platform identity by itself. Android integrations must explicitly opt in.

#![forbid(unsafe_code)]

mod generator;
mod scope;

pub use generator::{DerivationError, decode_seed_hex, generate_android_id, generate_app_instance_id};
pub use scope::{AppScope, ScopeError};

#[cfg(test)]
mod tests;
