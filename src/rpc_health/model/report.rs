use serde::Serialize;

use super::RpcHealthStatus;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RpcMethodHealth {
    pub method: &'static str,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RpcHealthReport {
    pub endpoint: String,
    pub status: RpcHealthStatus,
    pub version: Option<String>,
    pub block_count: Option<u64>,
    /// Whether the node reported it is still catching up. `None` where the
    /// family has no syncing call, or the call did not give a verdict.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syncing: Option<bool>,
    pub methods: Vec<RpcMethodHealth>,
}

impl RpcHealthReport {
    pub fn status_label(&self) -> &'static str {
        self.status.label()
    }

    pub fn message(&self) -> String {
        match self.status {
            RpcHealthStatus::Healthy => {
                let block_count = self
                    .block_count
                    .map_or_else(|| "unknown".to_string(), |value| value.to_string());
                let version = self.version.as_deref().unwrap_or("unknown version");
                format!("{version}; block-count {block_count}")
            }
            RpcHealthStatus::Degraded | RpcHealthStatus::Unreachable => {
                match self.methods.iter().find(|method| !method.ok) {
                    Some(method) => format!("{}: {}", method.method, method.detail),
                    None if self.syncing == Some(true) => {
                        "node is syncing; the chain it serves is not at head".to_string()
                    }
                    None => "RPC probe did not complete.".to_string(),
                }
            }
        }
    }

    pub fn to_cli_text(&self) -> String {
        let mut lines = vec![
            format!("rpc-health: {}", self.status.label()),
            format!("endpoint: {}", self.endpoint),
            format!("message: {}", self.message()),
        ];

        if let Some(version) = &self.version {
            lines.push(format!("version: {version}"));
        }
        if let Some(block_count) = self.block_count {
            lines.push(format!("block-count: {block_count}"));
        }
        if let Some(syncing) = self.syncing {
            lines.push(format!("syncing: {syncing}"));
        }
        for method in &self.methods {
            lines.push(format!(
                "method-{}: {}",
                method.method,
                if method.ok { "ok" } else { "failed" }
            ));
            lines.push(format!(
                "method-{}-detail: {}",
                method.method, method.detail
            ));
        }
        lines.push(String::new());
        lines.join("\n")
    }
}
