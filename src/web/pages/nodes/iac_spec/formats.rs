//! Per-target IaC string builders. Each function renders one node (or one
//! fleet) into the target format the operator pastes into a platform. Kept
//! apart from `IacFormat` dispatch and the in-page card so a format reads
//! beside nothing else.

use crate::{
    agents::HermesAgentAssociation, core::node::NodeConfig, roles::NodeRole, signing::SignerKeyRef,
};

pub(super) fn generate_cli_spec(
    node: &NodeConfig,
    role: Option<NodeRole>,
    signer: Option<&SignerKeyRef>,
    assoc: Option<&HermesAgentAssociation>,
) -> String {
    let role_slug = role
        .map(|r| r.slug().to_string())
        // "none", not "observer": a generated manifest that names a real duty
        // the node was never assigned would provision the wrong thing.
        .unwrap_or_else(|| "none".to_string());
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
        cmd.push_str(&format!(
            " --signer-backend {} --signer-key {}",
            s.backend_id, s.key_id
        ));
    }
    if assoc.is_some_and(|a| a.enabled) {
        cmd.push_str(" --hermes-enabled");
    }
    cmd
}

pub(super) fn generate_json_spec(
    node: &NodeConfig,
    role: Option<NodeRole>,
    signer: Option<&SignerKeyRef>,
    assoc: Option<&HermesAgentAssociation>,
) -> String {
    let role_slug = role
        .map(|r| r.slug().to_string())
        // "none", not "observer": a generated manifest that names a real duty
        // the node was never assigned would provision the wrong thing.
        .unwrap_or_else(|| "none".to_string());
    let role_label = role
        .map(|role| role.label().to_string())
        // Not "Node": a manifest that labels an unassigned node with a duty
        // name reads as an assignment nobody made.
        .unwrap_or_else(|| "No duty assigned".to_string());

    let signer_val = signer.map(|k| {
        serde_json::json!({
            "backend_id": k.backend_id,
            "key_id": k.key_id,
        })
    });

    let hermes_val = assoc.map(|a| {
        serde_json::json!({
            "enabled": a.enabled,
            "autonomous_healing": a.autonomous_healing,
            "agent_version": a.agent_version,
        })
    });

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
            // Served to any read-fleet credential and meant to be pasted into a
            // manifest, so a secret an operator put on the command line is
            // redacted rather than re-exported. Supply it through the target
            // platform's own secret mechanism.
            "args": crate::redaction::redact_sensitive_args(&node.args),
        }
    });
    serde_json::to_string_pretty(&json_val).unwrap_or_else(|_| "{}".to_string())
}

pub(super) fn generate_docker_spec(node: &NodeConfig) -> String {
    let client = node.node_type.to_string();
    let version = if node.runtime_version.is_empty() {
        "latest"
    } else {
        &node.runtime_version
    };
    let rpc = if node.rpc_port == 0 {
        10332
    } else {
        node.rpc_port
    };
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

pub(super) fn generate_k8s_spec(node: &NodeConfig, role: Option<NodeRole>) -> String {
    let role_slug = role
        .map(|r| r.slug().to_string())
        // "none", not "observer": a generated manifest that names a real duty
        // the node was never assigned would provision the wrong thing.
        .unwrap_or_else(|| "none".to_string());
    let client = node.node_type.to_string();
    let version = if node.runtime_version.is_empty() {
        "latest"
    } else {
        &node.runtime_version
    };
    let rpc = if node.rpc_port == 0 {
        10332
    } else {
        node.rpc_port
    };
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

pub(super) fn generate_cloudformation_spec(node: &NodeConfig, role: Option<NodeRole>) -> String {
    let role_slug = role
        .map(|r| r.slug().to_string())
        // "none", not "observer": a generated manifest that names a real duty
        // the node was never assigned would provision the wrong thing.
        .unwrap_or_else(|| "none".to_string());
    let client = node.node_type.to_string();
    let version = if node.runtime_version.is_empty() {
        "latest"
    } else {
        &node.runtime_version
    };
    let rpc = if node.rpc_port == 0 {
        10332
    } else {
        node.rpc_port
    };
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

pub(super) fn generate_terraform_spec(node: &NodeConfig, role: Option<NodeRole>) -> String {
    let role_slug = role
        .map(|r| r.slug().to_string())
        // "none", not "observer": a generated manifest that names a real duty
        // the node was never assigned would provision the wrong thing.
        .unwrap_or_else(|| "none".to_string());
    let client = node.node_type.to_string();
    let version = if node.runtime_version.is_empty() {
        "latest"
    } else {
        &node.runtime_version
    };
    let rpc = if node.rpc_port == 0 {
        10332
    } else {
        node.rpc_port
    };
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

pub fn generate_fleet_compose(nodes: &[NodeConfig]) -> String {
    let mut out = String::from("version: '3.8'\nservices:\n");
    for node in nodes {
        let client = node.node_type.to_string();
        let version = if node.runtime_version.is_empty() {
            "latest"
        } else {
            &node.runtime_version
        };
        let rpc = if node.rpc_port == 0 {
            10332
        } else {
            node.rpc_port
        };
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
        let version = if node.runtime_version.is_empty() {
            "latest"
        } else {
            &node.runtime_version
        };
        let rpc = if node.rpc_port == 0 {
            10332
        } else {
            node.rpc_port
        };
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
        let version = if node.runtime_version.is_empty() {
            "latest"
        } else {
            &node.runtime_version
        };
        let rpc = if node.rpc_port == 0 {
            10332
        } else {
            node.rpc_port
        };
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
