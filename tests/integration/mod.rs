//! Integration Test Suite Module
//! 
//! Comprehensive testing infrastructure for Node Manager covering all 5 node types,
//! metrics collection, log parsing, plugin workflows, and cross-module interactions.

pub mod common;
pub mod fixtures;
pub mod mocks;

// Main integration test file with full test coverage
pub mod node_manager_full;

/// Re-export key components for external test modules
pub use self::node_manager_full::*;
