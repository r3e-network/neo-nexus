//! Integration Tests Entry Point
//!
//! Cargo treats each `tests/*.rs` file as its own test crate, so the modular
//! suite under `tests/integration/` is mounted here. Leaving it unmounted made
//! this target a one-test stub while 68 `#[test]` functions sat uncompilable.

#[path = "integration/mod.rs"]
mod integration;
