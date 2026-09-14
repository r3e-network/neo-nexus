//! Node role & capability preset picker and cloud sizing flavors.

use crate::web::{html, node_form::NodeDraft};

pub struct PresetCard {
    pub key: &'static str,
    pub icon: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
    pub rpc_on: bool,
    pub plugins: &'static str,
    pub storage: Option<&'static str>,
    pub badge: &'static str,
    /// What a host running this duty typically needs. Guidance for the
    /// operator sizing their own machine — NeoNexus allocates nothing and
    /// `NewNode` has no resource field, so this must never be phrased as
    /// something the product provisions.
    pub sizing: &'static str,
}

pub const PRESETS: &[PresetCard] = &[
    PresetCard {
        key: "rpc-api",
        icon: "🌐",
        title: "RPC / API Gateway",
        desc: "Expose JSON-RPC 2.0 API for wallets, dApps, SDKs, and explorers.",
        rpc_on: true,
        plugins: "RpcServer",
        storage: None,
        badge: "RPC Active · API",
        sizing: "8 vCPU · 32 GB RAM · 1 Gbps",
    },
    PresetCard {
        key: "relay",
        icon: "🛡️",
        title: "P2P Gossip Relay",
        desc: "Gossip and propagate blocks/txs across P2P mesh. Zero RPC attack surface.",
        rpc_on: false,
        plugins: "",
        storage: None,
        badge: "RPC Off · Pure P2P",
        sizing: "2 vCPU · 4 GB RAM · Low Latency",
    },
    PresetCard {
        key: "validator",
        icon: "⚡",
        title: "Consensus Validator",
        desc: "dBFT consensus block production node. Hardened with RPC disabled by default.",
        rpc_on: false,
        plugins: "DBFTPlugin",
        storage: None,
        badge: "RPC Off · dBFT Node",
        sizing: "4 vCPU · 16 GB RAM · NVMe · Leased Signer",
    },
    PresetCard {
        key: "indexer",
        icon: "📊",
        title: "Data Indexer & Archive",
        desc: "Full contract execution trace logs and NEP-11/17 token transfer tracking.",
        rpc_on: true,
        plugins: "RpcServer,ApplicationLogs,TokensTracker,StateService",
        storage: Some("rocksdb"),
        badge: "RPC Active · RocksDB",
        sizing: "16 vCPU · 64 GB RAM · 2 TB NVMe",
    },
    PresetCard {
        key: "oracle",
        icon: "🔮",
        title: "Oracle Node",
        desc: "Execute HTTPS and NeoFS requests for on-chain smart contract oracle feeds.",
        rpc_on: true,
        plugins: "RpcServer,OracleService",
        storage: None,
        badge: "RPC Active · Oracle",
        sizing: "4 vCPU · 16 GB RAM · HTTPS Outbound",
    },
    PresetCard {
        key: "observer",
        icon: "👁️",
        title: "Chain Observer",
        desc: "Read-only node observing and syncing ledger state for monitoring.",
        rpc_on: true,
        plugins: "RpcServer",
        storage: None,
        badge: "RPC Active · Read-Only",
        sizing: "2 vCPU · 8 GB RAM · 500 GB SSD",
    },
    PresetCard {
        key: "custom",
        icon: "🛠️",
        title: "Custom Config",
        desc: "Manual capability, port, and extension configuration without presets.",
        rpc_on: true,
        plugins: "",
        storage: None,
        badge: "Custom Tuning",
        sizing: "Custom Instance Flavor",
    },
];

pub fn role_presets_picker(draft: &NodeDraft) -> String {
    // Deliberately no substitution. This read the empty string as `rpc-api`, so
    // reopening the editor on a node with no duty showed the RPC card selected
    // — and saving the form then assigned that duty to a node the operator had
    // never given one. An empty draft shows nothing selected, which is what it
    // means.
    let current_role = draft.role.trim();

    let cards = PRESETS
        .iter()
        .map(|p| {
            let active = if current_role == p.key { " active" } else { "" };
            let rpc_flag = if p.rpc_on { "1" } else { "0" };
            let storage_attr = p
                .storage
                .map_or_else(String::new, |s| format!(r#" data-storage="{s}""#));
            format!(
                r#"<button type="button" class="role-preset-card{active}" data-role="{key}" data-rpc="{rpc_flag}" data-plugins="{plugins}"{storage_attr} onclick="selectNodeRolePreset(this)">
                    <div class="role-card-top">
                        <span class="role-card-icon">{icon}</span>
                        <span class="role-card-badge">{badge}</span>
                    </div>
                    <div class="role-card-title">{title}</div>
                    <div class="role-card-desc">{desc}</div>
                    <div class="role-card-sizing" style="margin-top: 6px; font-size: 11px; opacity: 0.8; display: flex; align-items: center; gap: 4px;">
                        <span>Typical host:</span> <span class="mono">{sizing}</span>
                    </div>
                </button>"#,
                key = p.key,
                active = active,
                rpc_flag = rpc_flag,
                plugins = p.plugins,
                storage_attr = storage_attr,
                icon = p.icon,
                badge = p.badge,
                title = p.title,
                desc = p.desc,
                sizing = p.sizing,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<div class="role-presets-section">
            <div class="section-lead-label">
                <span class="lead-icon">🎯</span>
                <div>
                    <strong>Node Role & Capability Preset</strong>
                    <div class="help">Select a role to automatically configure recommended services, storage engines, and plugins.</div>
                </div>
            </div>
            <div class="role-presets-grid">
                {cards}
            </div>
            <input type="hidden" id="f-role" name="role" value="{current_role}">
        </div>"#,
        cards = cards,
        current_role = html::escape(current_role),
    )
}
