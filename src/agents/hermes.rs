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
    /// An enrolment the operator asked for, with autonomous healing granted.
    ///
    /// Only for the paths where the operator explicitly opted in. To stand in
    /// for an instance that has no association at all, use [`Self::unenrolled`]
    /// — this one would claim two grants the workspace never recorded.
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            enabled: true,
            autonomous_healing: true,
            last_heartbeat_unix: None,
            agent_version: "0.5.0".to_string(),
        }
    }

    /// What an instance with no agent association looks like: no copilot, and
    /// no standing permission to restart itself.
    pub fn unenrolled(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            enabled: false,
            autonomous_healing: false,
            last_heartbeat_unix: None,
            agent_version: String::new(),
        }
    }

    /// Whether this instance has an agent association at all, as opposed to the
    /// stand-in used when rendering one that does not.
    pub fn is_enrolled(&self) -> bool {
        !self.agent_version.is_empty()
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

# Capabilities granted to this agent, and to this instance only:
# - get_node_status (real-time health, block height, peer count, sync status, signer lease)
# - get_node_config (declarative IaC instance specification and role)
# - get_node_logs (this instance's observation tail)
# - restart_node (self-healing recovery; needs the autonomous-healing grant, and
#   is bounded by a circuit breaker at 5 restarts per hour)
# - stop_node (gracefully stop instance process and release ports)
# - start_node (supervised startup with launch barrier)
# - smoke_test_node (SRE binary smoke sweep and health diagnostic checks)
# - get_node_iac (export cloud launch template, K8s Pod YAML, or Docker spec)
#
# Deliberately NOT granted: take_snapshot writes a whole-workspace backup
# covering every other instance, so it stays an operator action. The token above
# is refused on every route outside /api/nodes/{node_id}/, including other
# instances' logs and the fleet-wide endpoints.
"#,
        node_name = node_name,
        safe_id = node_id.replace('-', "_"),
        mcp_url = mcp_url,
    )
}
