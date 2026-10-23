//! The instance summary ribbon and the details tab.

use crate::{core::node::NodeConfig, roles::NodeRole, signing::SignerKeyRef, web::html};

/// AWS EC2-styled top instance summary ribbon.
pub fn summary_banner(
    node: &NodeConfig,
    role: Option<NodeRole>,
    signer: Option<&SignerKeyRef>,
) -> String {
    let rpc_text = if node.rpc_port == 0 {
        "Disabled".to_string()
    } else {
        format!("http://127.0.0.1:{}", node.rpc_port)
    };
    let p2p_text = format!("127.0.0.1:{}", node.p2p_port);
    // `None` is not `Observer`. Observer is a real, separately selectable duty
    // with real plugin effects, and substituting it for "no duty assigned"
    // meant an operator who chose the "P2P Gossip Relay" preset — which
    // resolves to no duty at all — saw a node described as an Observer, then
    // found the editor pre-selecting a different duty again on reopen.
    let role_label = role.map_or("No duty assigned", NodeRole::label);
    let instance_type = format!(
        "{}-{}",
        node.node_type,
        role.map(|r| r.slug()).unwrap_or("node")
    );
    let signer_text = match signer {
        Some(s) => format!("{}/{}", s.backend_id, s.key_id),
        None => "Unassigned".to_string(),
    };

    let status_class = match node.status {
        crate::types::NodeStatus::Running => "running",
        crate::types::NodeStatus::Starting => "starting",
        crate::types::NodeStatus::Stopped => "stopped",
        crate::types::NodeStatus::Error => "danger",
    };

    // The two badges beside the status pill named a zone and a VPC — the same
    // strings for every node, on a product that runs every node as a local
    // child process. There is no zone and no VPC; what there is, is a client
    // and a chain, so that is what they carry.
    format!(
        r#"<div class="panel aws-summary-card" style="margin-bottom: 16px; padding: 16px 20px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; border-bottom: 1px solid var(--line); padding-bottom: 10px; flex-wrap: wrap; gap: 8px;">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <span class="status-dot {status_class}" style="width: 10px; height: 10px;"></span>
                    <h2 style="margin: 0; font-size: 16px;">Instance Summary: <span class="mono" style="color: var(--jade);">{id}</span> ({name})</h2>
                </div>
                <div style="display: flex; gap: 6px; align-items: center;">
                    <span class="badge {status_class}">● {status}</span>
                    <span class="badge" style="background: rgba(255,255,255,0.06);">{client}</span>
                    <span class="badge" style="background: rgba(255,255,255,0.06);">{network}</span>
                </div>
            </div>
            <div class="grid" style="grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 14px;">
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">Instance ID</div>
                    <div class="mono" style="font-size: 13px; font-weight: 500;">{id}</div>
                </div>
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">Instance Type</div>
                    <div style="font-size: 13px; font-weight: 500;">t3.{instance_type}</div>
                </div>
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">Role Profile</div>
                    <div style="font-size: 13px; font-weight: 500;">{role_label}</div>
                </div>
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">Public IPv4 / RPC</div>
                    <div class="mono" style="font-size: 12px;">{rpc_text}</div>
                </div>
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">Private IP / P2P</div>
                    <div class="mono" style="font-size: 12px;">{p2p_text}</div>
                </div>
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">IAM Signer Lease</div>
                    <div class="mono" style="font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="{signer_title}">{signer_text}</div>
                </div>
            </div>
        </div>"#,
        id = html::escape(&node.id),
        name = html::escape(&node.name),
        status_class = status_class,
        status = node.status.label(),
        client = html::escape(&node.node_type.to_string()),
        network = html::escape(&node.network.to_string()),
        instance_type = html::escape(&instance_type),
        role_label = html::escape(role_label),
        rpc_text = html::escape(&rpc_text),
        p2p_text = html::escape(&p2p_text),
        signer_title = html::escape(&signer_text),
        signer_text = html::escape(&signer_text),
    )
}

/// Tab 1: Details View (AWS EC2 General Details)
pub fn render_tab_details(node: &NodeConfig) -> String {
    let rpc_value = if node.rpc_port == 0 {
        "Disabled (Zero attack surface)".to_string()
    } else {
        node.rpc_port.to_string()
    };
    let ws_value = if node.rpc_port == 0 {
        "Disabled".to_string()
    } else {
        node.ws_port
            .map_or_else(|| "Not configured".to_string(), |p| p.to_string())
    };
    // The same redaction the launch log and the support bundle apply to this
    // exact expression. Node arguments are operator-supplied, so a `--password`
    // or `--wif` lands here, and this page is served to any session.
    let command = crate::argv::format_command(
        &node.binary_path,
        &crate::redaction::redact_sensitive_args(&node.args),
    );

    let facts = [
        ("Instance Identifier", node.id.clone()),
        ("Instance Name", node.name.clone()),
        ("AMI / Client Engine", node.node_type.to_string()),
        ("Network Topology", node.network.to_string()),
        ("Storage Engine Driver", node.storage_engine.to_string()),
        ("Binary Image Path", node.binary_path.display().to_string()),
        (
            "Client Release Version",
            if node.runtime_version.is_empty() {
                "native-latest".to_string()
            } else {
                node.runtime_version.clone()
            },
        ),
        ("JSON-RPC 2.0 Port", rpc_value),
        ("P2P Mesh Port", node.p2p_port.to_string()),
        ("WebSocket Port", ws_value),
        (
            "Host Process ID (PID)",
            node.pid
                .map_or_else(|| "Not running / Supervised".to_string(), |p| p.to_string()),
        ),
        // These read "x86_64 Native Sandbox" and "NeoNexus Workbench Daemon"
        // over a plain `Command::new(binary_path)`. There is no sandbox and no
        // hypervisor — the client runs as a child process with this user's
        // privileges — and on an arm64 host the architecture was also simply
        // wrong. That combination materially changes an operator's threat model
        // for an untrusted client binary, which is the one place a console must
        // not flatter itself.
        (
            "How it runs",
            format!(
                "child process of NeoNexus on this host, {}, with this user's privileges",
                std::env::consts::ARCH
            ),
        ),
    ];
    let rows = facts
        .iter()
        .map(|(label, value)| html::row(&[html::cell(label), html::cell(value)]))
        .collect::<Vec<_>>();
    let table = html::table(&["Attribute", "Configuration Value"], &rows);

    format!(
        r#"<div style="margin-top: 12px;">
            <h3>Instance Attributes</h3>
            {table}
            <h3 style="margin-top: 18px;">Host Process Launch Command</h3>
            {command}
        </div>"#,
        table = table,
        command = html::text_block(&command),
    )
}
