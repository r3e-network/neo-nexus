//! The config tab and the networking tab.

use crate::{
    config::ConfigGenerator,
    core::node::NodeConfig,
    web::{html, WebState},
};

/// The managed config, as text, redacted.
///
/// `/config` rendered a **path**. `ConfigGenerator::render_for_node` was never
/// called from `src/web/` at all — only drift, export and diagnostics used it —
/// so the file an operator needs to read while diagnosing a crash loop was the
/// one thing the console would not show them. The log-diagnosis remediation
/// button even routed back to `/config`, which showed the path again.
///
/// Redacted through the same helper `/logs` uses, because these configs embed
/// plaintext wallet unlock passwords.
pub fn render_tab_config(node: &NodeConfig, plugins: &[crate::catalog::PluginState]) -> String {
    let rendered = ConfigGenerator::render_for_node(node, plugins);
    let body = match rendered {
        Ok(rendered) => format!(
            r#"<p class="muted" style="font-size: 12px;">This is what NeoNexus writes for {name} — {format} — with secrets masked. It is written on every start, so an edit made by hand here is replaced the next time this node launches.</p>
{text}"#,
            name = html::escape(&node.name),
            format = html::escape(rendered.format.label()),
            text = html::text_block(&crate::redaction::redact_sensitive_text(&rendered.text)),
        ),
        Err(error) => html::notice(
            "warn",
            &format!(
                "This node's config cannot be rendered, which is also why it cannot start: {error:#}"
            ),
        ),
    };
    format!(
        r#"<div style="margin-top: 12px;"><h3>Managed configuration</h3>{body}<div style="margin-top: 12px;"><a class="btn small" href="/config">Every node's config</a></div></div>"#
    )
}

/// Tab 4: the ports, and what NeoNexus asks the client to do with them.
///
/// This was headed "Inbound Security Group Rules (Firewall Ruleset)" with a
/// "Rule Status: Open" column, over a product that has no firewall capability
/// whatsoever — `iptables`, `pfctl`, `ufw` and `nftables` appear in this
/// repository only inside UI captions. The CIDRs were not arbitrary: they
/// mirror the binds NeoNexus writes into the managed config. But they were
/// rendered as literals, so they went stale the moment an operator's own
/// arguments diverged — and a `--config` argument suppresses the managed
/// config entirely.
///
/// What the table can state truthfully is what NeoNexus *asks for*, and where
/// the client is free to ignore it.
pub fn render_tab_networking(state: &WebState, node: &NodeConfig) -> String {
    let overridden = node
        .args
        .iter()
        .any(|arg| arg == "--config" || arg.starts_with("--config=") || arg == "--config-file");

    let requested = |what: &str, port: Option<u16>, bind: &str, note: String| {
        html::row(&[
            html::cell(what),
            html::cell(&port.map_or_else(|| "—".to_string(), |port| port.to_string())),
            html::cell(bind),
            html::cell(&note),
        ])
    };

    let rpc_row = if node.rpc_port == 0 {
        requested(
            "JSON-RPC",
            None,
            "not requested",
            "this node exposes no RPC, so nothing in this workspace can ask it anything"
                .to_string(),
        )
    } else {
        requested(
            "JSON-RPC",
            Some(node.rpc_port),
            "127.0.0.1",
            "loopback only; reachable from this host".to_string(),
        )
    };
    let p2p_row = requested(
        "P2P",
        Some(node.p2p_port),
        "0.0.0.0",
        "all interfaces; peers must be able to reach it".to_string(),
    );
    let ws_row = match node.ws_port {
        // Honest about G23: a WS port is validated, reserved against other
        // nodes and rendered as an endpoint — and for every client except
        // neox-geth and the neo-cli sidecar, nothing ever opens it.
        Some(ws) if ws_is_requested(node) => requested(
            "WebSocket",
            Some(ws),
            "127.0.0.1",
            "loopback only".to_string(),
        ),
        Some(ws) => requested(
            "WebSocket",
            Some(ws),
            "not requested",
            format!(
                "reserved for this node, but NeoNexus emits no WebSocket setting for {}, so the port will not be listening",
                node.node_type
            ),
        ),
        None => requested("WebSocket", None, "not requested", "not configured".to_string()),
    };

    let table = html::table(
        &[
            "Service",
            "Port",
            "NeoNexus asks it to bind",
            "What that means",
        ],
        &[rpc_row, p2p_row, ws_row],
    );

    let caveat = if overridden {
        html::notice(
            "warn",
            "This node's arguments include --config, which makes the client read a file \
             NeoNexus did not write. The binds above are what NeoNexus would have asked for; \
             what the node is actually doing is in that file.",
        )
    } else {
        html::note(
            "These are the binds NeoNexus writes into the managed config. Arguments you add \
             pass to the client verbatim and can override any of them. NeoNexus opens no \
             firewall and closes none: the host's own rules decide what is reachable.",
        )
    };

    let endpoints = crate::web::pages::nodes::detail::endpoints_card(state, node);

    format!(
        r#"<div style="margin-top: 12px;">
            <h3>Ports</h3>
            {table}
            {caveat}
            <div style="margin-top: 16px;">
                {endpoints}
            </div>
        </div>"#
    )
}

/// Whether any launch path for this client emits a WebSocket setting.
///
/// neox-geth's generated config carries `ws_host`/`ws_port`, and the neo-cli
/// sidecar derives one. Nothing else does, and no `--ws` flag exists anywhere
/// in this repository — so for the remaining clients a configured WebSocket
/// port is a reservation, not a listener.
fn ws_is_requested(node: &NodeConfig) -> bool {
    matches!(
        node.node_type,
        crate::types::NodeType::NeoXGeth | crate::types::NodeType::NeoCli
    )
}
