//! The security, tags and IaC tabs.

use crate::web::pages::nodes::{binding::signer_binding, iac_spec::iac_spec_card};
use crate::{
    agents::HermesAgentAssociation,
    core::node::NodeConfig,
    roles::NodeRole,
    signing::SignerKeyRef,
    web::{html, WebState},
};

/// Tab 5: Security & IAM View
pub fn render_tab_security(
    state: &WebState,
    node: &NodeConfig,
    signer: Option<&SignerKeyRef>,
    new_token: Option<&str>,
) -> String {
    let signer_html = signer_binding(state, node, signer);
    let hermes_html = crate::web::pages::nodes::detail::hermes_agent_card(state, node, new_token);

    format!(
        r#"<div style="margin-top: 12px;">
            <div style="margin-bottom: 16px;">
                {signer_html}
            </div>
            <div>
                {hermes_html}
            </div>
        </div>"#,
        signer_html = signer_html,
        hermes_html = hermes_html,
    )
}

/// Tab 7: AWS Resource Tags
pub fn render_tab_tags(node: &NodeConfig, role: Option<NodeRole>) -> String {
    let rows = vec![
        html::row(&[html::cell("Name"), html::cell(&node.name)]),
        // "Environment: Production" was hardcoded for every node, testnet
        // included, under a claim that these drive cost allocation and access
        // control. There is no environment concept in the workspace and no way
        // to set one, so the row asserted something false about every node.
        html::row(&[html::cell("Network"), html::cell(&node.network.to_string())]),
        html::row(&[
            html::cell("Role"),
            html::cell(role.map_or("No duty assigned", NodeRole::label)),
        ]),
        html::row(&[
            html::cell("ClientEngine"),
            html::cell(&node.node_type.to_string()),
        ]),
        html::row(&[
            html::cell("ManagedBy"),
            html::cell("NeoNexus-CloudControlPlane"),
        ]),
        html::row(&[
            html::cell("InstanceId"),
            html::raw_cell(&format!(
                r#"<span class="mono">{}</span>"#,
                html::escape(&node.id)
            )),
        ]),
    ];
    let tag_table = html::table(&["Key", "Value"], &rows);

    format!(
        r#"<div style="margin-top: 12px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; flex-wrap: wrap; gap: 8px;">
                <div>
                    <h3 style="margin: 0 0 2px 0;">Resource Tags (Metadata Taxonomy)</h3>
                    <div class="muted" style="font-size: 12px;">Key-value pairs assigned to this instance for enterprise governance, cost allocation, and access control.</div>
                </div>
            </div>
            {tag_table}
        </div>"#,
        tag_table = tag_table,
    )
}

/// Tab 8: Launch Templates & IaC Export
pub fn render_tab_iac(
    node: &NodeConfig,
    role: Option<NodeRole>,
    signer: Option<&SignerKeyRef>,
    assoc: &HermesAgentAssociation,
) -> String {
    iac_spec_card(node, role, signer, Some(assoc))
}
