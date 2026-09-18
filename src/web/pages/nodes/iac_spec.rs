//! Cloud instance launch template, Infrastructure-as-Code (IaC) spec, container and Kubernetes manifests.

use std::str::FromStr;

use crate::{
    agents::HermesAgentAssociation, core::node::NodeConfig, roles::NodeRole, signing::SignerKeyRef,
    web::html,
};

mod formats;
pub use formats::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IacFormat {
    Json,
    K8s,
    Docker,
    Cli,
    CloudFormation,
    Terraform,
}

impl FromStr for IacFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "k8s" | "kubernetes" | "yaml" => Ok(Self::K8s),
            "docker" | "container" | "sh" => Ok(Self::Docker),
            "cli" | "cmd" | "txt" => Ok(Self::Cli),
            "cfn" | "cloudformation" | "aws" => Ok(Self::CloudFormation),
            "tf" | "terraform" | "hcl" => Ok(Self::Terraform),
            _ => Ok(Self::Json),
        }
    }
}

pub fn generate_node_iac(
    node: &NodeConfig,
    role: Option<NodeRole>,
    signer: Option<&SignerKeyRef>,
    assoc: Option<&HermesAgentAssociation>,
    format: IacFormat,
) -> (String, &'static str, &'static str) {
    match format {
        IacFormat::Json => (
            generate_json_spec(node, role, signer, assoc),
            "application/json",
            "json",
        ),
        IacFormat::K8s => (generate_k8s_spec(node, role), "application/x-yaml", "yaml"),
        IacFormat::Docker => (generate_docker_spec(node), "text/x-shellscript", "sh"),
        IacFormat::Cli => (
            generate_cli_spec(node, role, signer, assoc),
            "text/plain",
            "txt",
        ),
        IacFormat::CloudFormation => (
            generate_cloudformation_spec(node, role),
            "application/x-yaml",
            "cfn.yaml",
        ),
        IacFormat::Terraform => (
            generate_terraform_spec(node, role),
            "application/x-tf",
            "tf",
        ),
    }
}

pub fn iac_spec_card(
    node: &NodeConfig,
    role: Option<NodeRole>,
    signer: Option<&SignerKeyRef>,
    assoc: Option<&HermesAgentAssociation>,
) -> String {
    let cli_cmd = generate_cli_spec(node, role, signer, assoc);
    let json_spec = generate_json_spec(node, role, signer, assoc);
    let docker_cmd = generate_docker_spec(node);
    let k8s_yaml = generate_k8s_spec(node, role);
    let cfn_yaml = generate_cloudformation_spec(node, role);
    let tf_hcl = generate_terraform_spec(node, role);

    let safe_name = node.name.to_lowercase().replace(' ', "-");
    let enc_id = html::urlencoding_lite(&node.id);

    format!(
        r#"<div class="panel" style="margin-top: 16px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; flex-wrap: wrap; gap: 8px;">
                <div style="display: flex; align-items: center; gap: 8px;">
                    <span style="font-size: 18px;">☁️</span>
                    <h3 style="margin: 0;">Cloud Launch Template & Multi-Target IaC Spec</h3>
                </div>
                <div style="display: flex; gap: 6px;">
                    <span class="badge" style="background: rgba(255,255,255,0.06); font-family: monospace;">IaC v1.3</span>
                    <span class="badge running">Multi-Cloud Ready</span>
                </div>
            </div>
            <p class="muted" style="font-size: 13px; margin-bottom: 12px;">
                Declarative instance configuration export. Reproduce or provision this exact virtual instance configuration across AWS, Kubernetes, Docker, Terraform, or GitOps pipelines.
            </p>
            <div style="display: flex; gap: 8px; margin-bottom: 12px; flex-wrap: wrap;">
                <a href="/api/nodes/{id}/iac?format=cloudformation" download="node-{safe_name}-cfn.yaml" class="btn small" style="text-decoration:none;">📥 Download CloudFormation</a>
                <a href="/api/nodes/{id}/iac?format=terraform" download="node-{safe_name}.tf" class="btn small" style="text-decoration:none;">📥 Download Terraform</a>
                <a href="/api/nodes/{id}/iac?format=k8s" download="node-{safe_name}-k8s.yaml" class="btn small" style="text-decoration:none;">📥 Download K8s YAML</a>
                <a href="/api/nodes/{id}/iac?format=docker" download="node-{safe_name}-docker.sh" class="btn small" style="text-decoration:none;">📥 Download Docker Script</a>
                <a href="/api/nodes/{id}/iac?format=json" download="node-{safe_name}-spec.json" class="btn small" style="text-decoration:none;">📥 Download JSON Spec</a>
            </div>
            <details class="panel" style="background: rgba(0,0,0,0.2); margin-bottom: 10px;">
                <summary style="cursor: pointer; font-weight: 500;">☁️ AWS CloudFormation Template (YAML)</summary>
                <div style="margin-top: 8px; display: flex; justify-content: flex-end; margin-bottom: 4px;">
                    <button type="button" class="btn small" data-copy-target="iac-cfn-code">📋 Copy CloudFormation</button>
                </div>
                <pre style="max-height: 240px; overflow-y: auto;"><code id="iac-cfn-code">{cfn_yaml}</code></pre>
            </details>
            <details class="panel" style="background: rgba(0,0,0,0.2); margin-bottom: 10px;">
                <summary style="cursor: pointer; font-weight: 500;">🏗️ Terraform HCL Specification (main.tf)</summary>
                <div style="margin-top: 8px; display: flex; justify-content: flex-end; margin-bottom: 4px;">
                    <button type="button" class="btn small" data-copy-target="iac-tf-code">📋 Copy Terraform</button>
                </div>
                <pre style="max-height: 240px; overflow-y: auto;"><code id="iac-tf-code">{tf_hcl}</code></pre>
            </details>
            <details class="panel" style="background: rgba(0,0,0,0.2); margin-bottom: 10px;">
                <summary style="cursor: pointer; font-weight: 500;">☸️ Kubernetes Cloud Pod Spec (YAML)</summary>
                <div style="margin-top: 8px; display: flex; justify-content: flex-end; margin-bottom: 4px;">
                    <button type="button" class="btn small" data-copy-target="iac-k8s-code">📋 Copy K8s YAML</button>
                </div>
                <pre style="max-height: 240px; overflow-y: auto;"><code id="iac-k8s-code">{k8s_yaml}</code></pre>
            </details>
            <details class="panel" style="background: rgba(0,0,0,0.2); margin-bottom: 10px;">
                <summary style="cursor: pointer; font-weight: 500;">🐳 Docker Container Launch Spec</summary>
                <div style="margin-top: 8px; display: flex; justify-content: flex-end; margin-bottom: 4px;">
                    <button type="button" class="btn small" data-copy-target="iac-docker-code">📋 Copy Docker</button>
                </div>
                <pre style="max-height: 180px; overflow-y: auto;"><code id="iac-docker-code">{docker_cmd}</code></pre>
            </details>
            <details class="panel" style="background: rgba(0,0,0,0.2); margin-bottom: 10px;">
                <summary style="cursor: pointer; font-weight: 500;">📄 Declarative Cloud Instance Spec (JSON)</summary>
                <div style="margin-top: 8px; display: flex; justify-content: flex-end; margin-bottom: 4px;">
                    <button type="button" class="btn small" data-copy-target="iac-json-code">📋 Copy JSON</button>
                </div>
                <pre style="max-height: 240px; overflow-y: auto;"><code id="iac-json-code">{json_spec}</code></pre>
            </details>
            <details class="panel" style="background: rgba(0,0,0,0.2);">
                <summary style="cursor: pointer; font-weight: 500;">💻 Equivalent CLI Command</summary>
                <div style="margin-top: 8px; display: flex; gap: 8px;">
                    <input class="mono" readonly value="{cli_cmd}" style="flex: 1; padding: 6px 10px; background: var(--bg-card); border: 1px solid var(--line);" id="iac-cli-box">
                    <button type="button" class="btn small" data-copy-target="iac-cli-box">📋 Copy</button>
                </div>
            </details>
        </div>"#,
        id = enc_id,
        safe_name = html::escape(&safe_name),
        cfn_yaml = html::escape(&cfn_yaml),
        tf_hcl = html::escape(&tf_hcl),
        cli_cmd = html::escape(&cli_cmd),
        json_spec = html::escape(&json_spec),
        docker_cmd = html::escape(&docker_cmd),
        k8s_yaml = html::escape(&k8s_yaml),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Network, NodeType, StorageEngine};
    use std::path::PathBuf;

    fn test_node() -> NodeConfig {
        NodeConfig {
            id: "node-iac-test-1".to_string(),
            name: "validator-iac".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Mainnet,
            binary_path: PathBuf::from("/opt/neo-go"),
            args: vec!["--config".to_string(), "config.yml".to_string()],
            runtime_version: "v0.106.0".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: Some(10334),
            status: crate::types::NodeStatus::Stopped,
            pid: None,
        }
    }

    #[test]
    fn iac_spec_generates_all_targets_with_role_and_signer() {
        let node = test_node();
        let role = Some(NodeRole::Consensus);
        let signer = SignerKeyRef::new("vault-profile", "key-01").ok();
        let assoc = HermesAgentAssociation::new(&node.id);

        let html = iac_spec_card(&node, role, signer.as_ref(), Some(&assoc));

        // CloudFormation check
        assert!(html.contains("AWSTemplateFormatVersion: &#39;2010-09-09&#39;"));
        assert!(html.contains("AWS::ECS::TaskDefinition"));

        // Terraform check
        assert!(html.contains("required_providers"));
        assert!(html.contains("docker_container"));

        // CLI command check
        assert!(html.contains("neo-nexus node add --name validator-iac"));
        assert!(html.contains("--role validator"));
        assert!(html.contains("--signer-backend vault-profile --signer-key key-01"));
        assert!(html.contains("--hermes-enabled"));

        // JSON check
        assert!(html.contains("neonexus.io/v1alpha1"));
        assert!(html.contains("NodeInstance"));
        assert!(html.contains("iam_signer_lease"));

        // Docker check
        assert!(html.contains("docker run -d"));
        assert!(html.contains("-p 10333:10333"));
        assert!(html.contains("neonexus/neo-go:v0.106.0"));

        // K8s check
        assert!(html.contains("apiVersion: v1"));
        assert!(html.contains("kind: Pod"));
        assert!(html.contains("neonexus.io/role"));

        // Download & Copy triggers
        assert!(html.contains("Download CloudFormation"));
        assert!(html.contains("Download Terraform"));
        assert!(html.contains("Download K8s YAML"));
        assert!(html.contains("data-copy-target"));
    }

    #[test]
    fn discrete_generators_and_format_dispatch() {
        let node = test_node();
        let (spec, mime, ext) = generate_node_iac(&node, None, None, None, IacFormat::K8s);
        assert_eq!(mime, "application/x-yaml");
        assert_eq!(ext, "yaml");
        assert!(spec.contains("kind: Pod"));

        let (spec_json, mime_json, ext_json) =
            generate_node_iac(&node, None, None, None, IacFormat::Json);
        assert_eq!(mime_json, "application/json");
        assert_eq!(ext_json, "json");
        assert!(spec_json.contains("neonexus.io/v1alpha1"));

        let (spec_cfn, mime_cfn, ext_cfn) =
            generate_node_iac(&node, None, None, None, IacFormat::CloudFormation);
        assert_eq!(mime_cfn, "application/x-yaml");
        assert_eq!(ext_cfn, "cfn.yaml");
        assert!(spec_cfn.contains("AWSTemplateFormatVersion"));

        let (spec_tf, mime_tf, ext_tf) =
            generate_node_iac(&node, None, None, None, IacFormat::Terraform);
        assert_eq!(mime_tf, "application/x-tf");
        assert_eq!(ext_tf, "tf");
        assert!(spec_tf.contains("terraform {"));

        let fleet = vec![node.clone()];
        let compose = generate_fleet_compose(&fleet);
        assert!(compose.contains("version: '3.8'"));
        assert!(compose.contains("validator-iac:"));

        let k8s_fleet = generate_fleet_k8s(&fleet);
        assert!(k8s_fleet.contains("kind: Pod"));

        let cfn_fleet = generate_fleet_cloudformation(&fleet);
        assert!(cfn_fleet.contains("AWSTemplateFormatVersion"));

        let tf_fleet = generate_fleet_terraform(&fleet);
        assert!(tf_fleet.contains("resource \"docker_container\""));
    }
}
