//! Cloud instance launch template, Infrastructure-as-Code (IaC) spec, container and Kubernetes manifests.

use std::str::FromStr;

use crate::{
    agents::HermesAgentAssociation,
    core::node::NodeConfig,
    roles::NodeRole,
    signing::SignerKeyRef,
    web::html,
};

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

pub fn generate_cli_spec(
    node: &NodeConfig,
    role: Option<NodeRole>,
    signer: Option<&SignerKeyRef>,
    assoc: Option<&HermesAgentAssociation>,
) -> String {
    let role_slug = role.map(|r| r.slug().to_string()).unwrap_or_else(|| "observer".to_string());
    let mut cmd = format!(
        "neo-nexus node add --name {} --type {} --network {} --p2p-port {} --rpc-port {} --storage {} --role {}",
        node.name,
        node.node_type,
        node.network,
        node.p2p_port,
        node.rpc_port,
        node.storage_engine,
        role_slug,
    );
    if let Some(ws) = node.ws_port {
        cmd.push_str(&format!(" --ws-port {ws}"));
    }
    if let Some(s) = signer {
        cmd.push_str(&format!(" --signer-backend {} --signer-key {}", s.backend_id, s.key_id));
    }
    if assoc.is_some_and(|a| a.enabled) {
        cmd.push_str(" --hermes-enabled");
    }
    cmd
}

pub fn generate_json_spec(
    node: &NodeConfig,
    role: Option<NodeRole>,
    signer: Option<&SignerKeyRef>,
    assoc: Option<&HermesAgentAssociation>,
) -> String {
    let role_slug = role.map(|r| r.slug().to_string()).unwrap_or_else(|| "observer".to_string());
    let role_label = role.map(|r| r.label().to_string()).unwrap_or_else(|| "Node".to_string());

    let signer_val = signer.map(|k| serde_json::json!({
        "backend_id": k.backend_id,
        "key_id": k.key_id,
    }));

    let hermes_val = assoc.map(|a| serde_json::json!({
        "enabled": a.enabled,
        "autonomous_healing": a.autonomous_healing,
        "agent_version": a.agent_version,
    }));

    let json_val = serde_json::json!({
        "schema": "neonexus.io/v1alpha1",
        "kind": "NodeInstance",
        "metadata": {
            "id": node.id,
            "name": node.name,
            "role": role_slug,
            "role_label": role_label,
        },
        "spec": {
            "client": node.node_type.to_string(),
            "network": node.network.to_string(),
            "runtime_version": node.runtime_version,
            "storage_engine": node.storage_engine.to_string(),
            "ports": {
                "p2p": node.p2p_port,
                "rpc": if node.rpc_port == 0 { None } else { Some(node.rpc_port) },
                "ws": node.ws_port,
            },
            "iam_signer_lease": signer_val,
            "hermes_copilot": hermes_val,
            "binary_path": node.binary_path.display().to_string(),
            "args": node.args,
        }
    });
    serde_json::to_string_pretty(&json_val).unwrap_or_else(|_| "{}".to_string())
}

pub fn generate_docker_spec(node: &NodeConfig) -> String {
    let client = node.node_type.to_string();
    let version = if node.runtime_version.is_empty() { "latest" } else { &node.runtime_version };
    let rpc = if node.rpc_port == 0 { 10332 } else { node.rpc_port };
    let safe_name = node.name.to_lowercase().replace(' ', "-");
    format!(
        "docker run -d \\\n  --name {safe_name} \\\n  --restart unless-stopped \\\n  -p {p2p}:{p2p} \\\n  -p {rpc}:{rpc} \\\n  -v /var/lib/neonexus/{safe_name}:/data \\\n  neonexus/{client}:{version}",
        safe_name = safe_name,
        p2p = node.p2p_port,
        rpc = rpc,
        client = client,
        version = version,
    )
}

pub fn generate_k8s_spec(node: &NodeConfig, role: Option<NodeRole>) -> String {
    let role_slug = role.map(|r| r.slug().to_string()).unwrap_or_else(|| "observer".to_string());
    let client = node.node_type.to_string();
    let version = if node.runtime_version.is_empty() { "latest" } else { &node.runtime_version };
    let rpc = if node.rpc_port == 0 { 10332 } else { node.rpc_port };
    let safe_name = node.name.to_lowercase().replace(' ', "-");
    format!(
r#"apiVersion: v1
kind: Pod
metadata:
  name: {name}
  labels:
    app.kubernetes.io/name: neonexus-node
    neonexus.io/instance-id: "{id}"
    neonexus.io/role: "{role}"
    neonexus.io/network: "{network}"
spec:
  restartPolicy: Always
  containers:
    - name: {client}
      image: neonexus/{client}:{version}
      ports:
        - containerPort: {p2p}
          name: p2p
          protocol: TCP
        - containerPort: {rpc}
          name: rpc
          protocol: TCP
      volumeMounts:
        - mountPath: /data
          name: node-data
  volumes:
    - name: node-data
      persistentVolumeClaim:
        claimName: {name}-pvc"#,
        name = safe_name,
        id = node.id,
        role = role_slug,
        network = node.network,
        client = client,
        version = version,
        p2p = node.p2p_port,
        rpc = rpc,
    )
}

pub fn generate_cloudformation_spec(node: &NodeConfig, role: Option<NodeRole>) -> String {
    let role_slug = role.map(|r| r.slug().to_string()).unwrap_or_else(|| "observer".to_string());
    let client = node.node_type.to_string();
    let version = if node.runtime_version.is_empty() { "latest" } else { &node.runtime_version };
    let rpc = if node.rpc_port == 0 { 10332 } else { node.rpc_port };
    let safe_name = node.name.to_lowercase().replace(' ', "-");
    format!(
r#"AWSTemplateFormatVersion: '2010-09-09'
Description: AWS CloudFormation Template for NeoNexus Instance - {name}
Parameters:
  Environment:
    Type: String
    Default: production
    AllowedValues: [production, staging, testnet]
Resources:
  NodeTaskDefinition:
    Type: AWS::ECS::TaskDefinition
    Properties:
      Family: neonexus-{name}
      NetworkMode: awsvpc
      RequiresCompatibilities: [FARGATE, EC2]
      Cpu: '1024'
      Memory: '2048'
      ContainerDefinitions:
        - Name: {client}
          Image: neonexus/{client}:{version}
          Essential: true
          PortMappings:
            - ContainerPort: {p2p}
              HostPort: {p2p}
              Protocol: tcp
            - ContainerPort: {rpc}
              HostPort: {rpc}
              Protocol: tcp
          Environment:
            - Name: NEONEXUS_NODE_ID
              Value: "{id}"
            - Name: NEONEXUS_ROLE
              Value: "{role}"
            - Name: NEONEXUS_NETWORK
              Value: "{network}"
          MountPoints:
            - SourceVolume: node-data
              ContainerPath: /data
      Volumes:
        - Name: node-data
Outputs:
  InstanceId:
    Description: NeoNexus Virtual Instance Identifier
    Value: "{id}"
  RpcEndpoint:
    Description: Node RPC Interface
    Value: !Sub "http://${{AWS::StackName}}:{rpc}""#,
        name = safe_name,
        id = node.id,
        role = role_slug,
        network = node.network,
        client = client,
        version = version,
        p2p = node.p2p_port,
        rpc = rpc,
    )
}

pub fn generate_terraform_spec(node: &NodeConfig, role: Option<NodeRole>) -> String {
    let role_slug = role.map(|r| r.slug().to_string()).unwrap_or_else(|| "observer".to_string());
    let client = node.node_type.to_string();
    let version = if node.runtime_version.is_empty() { "latest" } else { &node.runtime_version };
    let rpc = if node.rpc_port == 0 { 10332 } else { node.rpc_port };
    let safe_name = node.name.to_lowercase().replace([' ', '-'], "_");
    format!(
r#"# Terraform HCL Definition for NeoNexus Instance: {name}
terraform {{
  required_version = ">= 1.5.0"
  required_providers {{
    docker = {{
      source  = "kreuzwerker/docker"
      version = "~> 3.0"
    }}
  }}
}}

resource "docker_volume" "{name}_data" {{
  name = "neonexus_{name}_data"
}}

resource "docker_container" "{name}" {{
  name  = "neonexus_{name}"
  image = "neonexus/{client}:{version}"
  restart = "unless-stopped"

  ports {{
    internal = {p2p}
    external = {p2p}
  }}

  ports {{
    internal = {rpc}
    external = {rpc}
  }}

  volumes {{
    volume_name    = docker_volume.{name}_data.name
    container_path = "/data"
  }}

  env = [
    "NEONEXUS_NODE_ID={id}",
    "NEONEXUS_ROLE={role}",
    "NEONEXUS_NETWORK={network}",
  ]

  labels = {{
    "neonexus.io/managed-by"  = "neo-nexus"
    "neonexus.io/instance-id" = "{id}"
    "neonexus.io/role"        = "{role}"
  }}
}}"#,
        name = safe_name,
        id = node.id,
        role = role_slug,
        network = node.network,
        client = client,
        version = version,
        p2p = node.p2p_port,
        rpc = rpc,
    )
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
        IacFormat::K8s => (
            generate_k8s_spec(node, role),
            "application/x-yaml",
            "yaml",
        ),
        IacFormat::Docker => (
            generate_docker_spec(node),
            "text/x-shellscript",
            "sh",
        ),
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

pub fn generate_fleet_compose(nodes: &[NodeConfig]) -> String {
    let mut out = String::from("version: '3.8'\nservices:\n");
    for node in nodes {
        let client = node.node_type.to_string();
        let version = if node.runtime_version.is_empty() { "latest" } else { &node.runtime_version };
        let rpc = if node.rpc_port == 0 { 10332 } else { node.rpc_port };
        let safe_name = node.name.to_lowercase().replace(' ', "-");
        out.push_str(&format!(
            "  {name}:\n    image: neonexus/{client}:{version}\n    container_name: {name}\n    restart: unless-stopped\n    ports:\n      - \"{p2p}:{p2p}\"\n      - \"{rpc}:{rpc}\"\n    volumes:\n      - {name}-data:/data\n",
            name = safe_name,
            client = client,
            version = version,
            p2p = node.p2p_port,
            rpc = rpc,
        ));
    }
    out.push_str("volumes:\n");
    for node in nodes {
        let safe_name = node.name.to_lowercase().replace(' ', "-");
        out.push_str(&format!("  {safe_name}-data:\n"));
    }
    out
}

pub fn generate_fleet_k8s(nodes: &[NodeConfig]) -> String {
    let mut docs = Vec::new();
    for node in nodes {
        docs.push(generate_k8s_spec(node, None));
    }
    docs.join("\n---\n")
}

pub fn generate_fleet_cloudformation(nodes: &[NodeConfig]) -> String {
    let mut out = String::from("AWSTemplateFormatVersion: '2010-09-09'\nDescription: AWS CloudFormation Cluster Manifest for NeoNexus Fleet\nResources:\n");
    for node in nodes {
        let client = node.node_type.to_string();
        let version = if node.runtime_version.is_empty() { "latest" } else { &node.runtime_version };
        let rpc = if node.rpc_port == 0 { 10332 } else { node.rpc_port };
        let safe_title = node.name.replace([' ', '-', '_'], "");
        let safe_name = node.name.to_lowercase().replace(' ', "-");
        out.push_str(&format!(
            "  {safe_title}Task:\n    Type: AWS::ECS::TaskDefinition\n    Properties:\n      Family: neonexus-{safe_name}\n      NetworkMode: awsvpc\n      RequiresCompatibilities: [FARGATE, EC2]\n      Cpu: '1024'\n      Memory: '2048'\n      ContainerDefinitions:\n        - Name: {client}\n          Image: neonexus/{client}:{version}\n          Essential: true\n          PortMappings:\n            - ContainerPort: {p2p}\n              HostPort: {p2p}\n              Protocol: tcp\n            - ContainerPort: {rpc}\n              HostPort: {rpc}\n              Protocol: tcp\n          Environment:\n            - Name: NEONEXUS_NODE_ID\n              Value: \"{id}\"\n",
            safe_title = safe_title,
            safe_name = safe_name,
            client = client,
            version = version,
            p2p = node.p2p_port,
            rpc = rpc,
            id = node.id,
        ));
    }
    out
}

pub fn generate_fleet_terraform(nodes: &[NodeConfig]) -> String {
    let mut out = String::from("# Terraform Fleet Cluster Manifest for NeoNexus\nterraform {\n  required_version = \">= 1.5.0\"\n  required_providers {\n    docker = {\n      source  = \"kreuzwerker/docker\"\n      version = \"~> 3.0\"\n    }\n  }\n}\n");
    for node in nodes {
        let client = node.node_type.to_string();
        let version = if node.runtime_version.is_empty() { "latest" } else { &node.runtime_version };
        let rpc = if node.rpc_port == 0 { 10332 } else { node.rpc_port };
        let safe_name = node.name.to_lowercase().replace([' ', '-'], "_");
        out.push_str(&format!(
            "\nresource \"docker_volume\" \"{name}_data\" {{\n  name = \"neonexus_{name}_data\"\n}}\n\nresource \"docker_container\" \"{name}\" {{\n  name  = \"neonexus_{name}\"\n  image = \"neonexus/{client}:{version}\"\n  restart = \"unless-stopped\"\n  ports {{\n    internal = {p2p}\n    external = {p2p}\n  }}\n  ports {{\n    internal = {rpc}\n    external = {rpc}\n  }}\n  volumes {{\n    volume_name    = docker_volume.{name}_data.name\n    container_path = \"/data\"\n  }}\n  env = [\n    \"NEONEXUS_NODE_ID={id}\",\n    \"NEONEXUS_NETWORK={network}\",\n  ]\n}}\n",
            name = safe_name,
            client = client,
            version = version,
            p2p = node.p2p_port,
            rpc = rpc,
            id = node.id,
            network = node.network,
        ));
    }
    out
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
    use std::path::PathBuf;
    use crate::types::{Network, NodeType, StorageEngine};

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

        let (spec_json, mime_json, ext_json) = generate_node_iac(&node, None, None, None, IacFormat::Json);
        assert_eq!(mime_json, "application/json");
        assert_eq!(ext_json, "json");
        assert!(spec_json.contains("neonexus.io/v1alpha1"));

        let (spec_cfn, mime_cfn, ext_cfn) = generate_node_iac(&node, None, None, None, IacFormat::CloudFormation);
        assert_eq!(mime_cfn, "application/x-yaml");
        assert_eq!(ext_cfn, "cfn.yaml");
        assert!(spec_cfn.contains("AWSTemplateFormatVersion"));

        let (spec_tf, mime_tf, ext_tf) = generate_node_iac(&node, None, None, None, IacFormat::Terraform);
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
