//! Integration Tests Entry Point
//!
//! Cargo treats each `tests/*.rs` file as its own test crate, so the modular
//! suite under `tests/integration/` is mounted here. It used to be a one-test
//! print stub while 68 `#[test]` functions sat uncompiled beside it; most of
//! those, once compiled, exercised a test-local fake rather than the product
//! and were replaced. `tests/integration/README.md` states what is left.

#[path = "integration/mod.rs"]
mod integration;
