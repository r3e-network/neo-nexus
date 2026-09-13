//! Hermes Agent association, telemetry, and scoped MCP configuration.
//!
//! Provides Nous Research Hermes Agent integration for autonomous node copilot
//! supervision, real-time metrics, and self-healing.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HermesAgentAssociation {
    pub node_id: String,
    pub enabled: bool,
    pub autonomous_healing: bool,
    pub last_heartbeat_unix: Option<i64>,
    pub agent_version: String,
}

impl HermesAgentAssociation {
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            enabled: true,
            autonomous_healing: true,
            last_heartbeat_unix: None,
            agent_version: "0.5.0".to_string(),
        }
    }

    pub fn is_alive(&self, current_unix: i64) -> bool {
        self.enabled
            && self
                .last_heartbeat_unix
                .is_some_and(|last| current_unix.saturating_sub(last) < 120)
    }
}

/// Generates a ready-to-use Nous Research Hermes config.yaml snippet with scoped MCP transport.
pub fn generate_hermes_config_snippet(node_id: &str, node_name: &str, mcp_url: &str) -> String {
    format!(
        r#"# Hermes Agent MCP Configuration for Node: {node_name}
# Add this to your Hermes profile config.yaml:
mcp_servers:
  neonexus_{safe_id}:
    transport: http
    url: "{mcp_url}"
    headers:
      Authorization: "Bearer ${{NEONEXUS_AGENT_TOKEN}}"
    connect_timeout: 10
    tool_timeout: 30

# Scoped capabilities granted to Hermes:
# - get_node_status (real-time health, block height, peer count, sync status, signer lease)
# - get_node_config (declarative IaC instance specification and role)
# - get_node_logs (real-time observation tail)
# - restart_node (autonomous self-healing recovery with circuit breaker)
# - stop_node (gracefully stop instance process and release ports)
# - start_node (supervised startup with launch barrier)
# - take_snapshot (pre-upgrade safety snapshot)
# - smoke_test_node (SRE binary smoke sweep and health diagnostic checks)
# - get_node_iac (export cloud launch template, K8s Pod YAML, or Docker spec)
"#,
        node_name = node_name,
        safe_id = node_id.replace('-', "_"),
        mcp_url = mcp_url,
    )
}
