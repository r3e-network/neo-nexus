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
                self.methods.iter().find(|method| !method.ok).map_or_else(
                    || "RPC probe did not complete.".to_string(),
                    |method| format!("{}: {}", method.method, method.detail),
                )
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
