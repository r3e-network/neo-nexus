use std::path::PathBuf;

use serde::Serialize;

use crate::redaction::redact_sensitive_text;

use super::super::CommitteeSidecarProcess;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrivateNetworkLaunchPackSidecarReport {
    pub root_path: PathBuf,
    pub manifest_path: PathBuf,
    pub sidecar_count: usize,
    pub sidecars: Vec<CommitteeSidecarProcess>,
}

impl PrivateNetworkLaunchPackSidecarReport {
    pub fn status_label(&self) -> &'static str {
        if self.sidecars.is_empty() {
            "empty"
        } else {
            "planned"
        }
    }

    pub fn to_cli_text(&self) -> String {
        let mut text = format!(
            "launch-pack-sidecars: {}\nmanifest: {}\nroot: {}\nsidecars: {}\n",
            self.status_label(),
            self.manifest_path.display(),
            self.root_path.display(),
            self.sidecar_count
        );

        if self.sidecars.is_empty() {
            text.push_str("sidecar: none\n");
            return text;
        }

        for sidecar in &self.sidecars {
            text.push_str(&format!(
                "sidecar: {} | {} | {}\n",
                sidecar.process.id, sidecar.signer_label, sidecar.process.kind
            ));
            text.push_str(&format!(
                "  binary: {}\n  args: {}\n  working-dir: {}\n  log: {}\n",
                sidecar.process.binary_path.display(),
                sidecar.process.args.len(),
                sidecar.process.working_dir.display(),
                sidecar.log_path.display()
            ));
            text.push_str(&format!(
                "  command: {}\n",
                redact_sensitive_text(&sidecar.process.display_command)
            ));
            if let Some(wallet_path) = &sidecar.wallet_path {
                text.push_str(&format!("  wallet: {}\n", wallet_path.display()));
            }
            if let Some(endpoint) = &sidecar.signer_endpoint {
                text.push_str(&format!("  endpoint: {endpoint}\n"));
            }
        }

        text
    }
}
