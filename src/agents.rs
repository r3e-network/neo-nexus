//! Autonomous Node Copilots and Guest Agent Integrations.
//!
//! Exposes Nous Research Hermes Agent supervision, telemetry, and scoped MCP
//! endpoints for autonomous blockchain node management.

pub mod hermes;

pub use hermes::{generate_hermes_config_snippet, HermesAgentAssociation};
