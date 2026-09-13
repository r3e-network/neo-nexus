//! Nodes: the fleet list, the per-node studio, and the delete confirmation.
//!
//! The list is the manager's front door, so it carries the actions an operator
//! came for — add, edit, delete — instead of only linking onward. Lifecycle
//! controls post to the same core pipeline the CLI drives, and deletion is a
//! two-step flow because nothing here can undo it.

mod activity;
mod binding;
mod delete;
mod detail;
mod detail_tabs;
mod iac_spec;
mod list;

pub use activity::instance_activity_card;
pub use binding::{save_signer_binding, SignerBindingForm};
pub use delete::{delete, delete_form};
pub use detail::{node_detail, provision_hermes_token, test_hermes_ping, toggle_hermes_healing};
pub use iac_spec::{
    generate_fleet_cloudformation, generate_fleet_compose, generate_fleet_k8s,
    generate_fleet_terraform, generate_node_iac, iac_spec_card, IacFormat,
};
pub use list::{node_list, resolve_density, NodeListQuery};

use crate::types::NodeConfig;

/// Explain why a control needs the instance stopped, and offer the stop.
///
/// Some settings can only change while the process is down — a running node has
/// already read its config and holds its ports. That is a real constraint and
/// stays. What did not need to stay was the dead end: three surfaces told the
/// operator "stop and settle the node first" and left them to find the control
/// themselves, on another page, and then navigate back.
pub(crate) fn stop_first(node: &NodeConfig, reason: &str) -> String {
    format!(
        r#"<div class="notice">
            <div>{reason}</div>
            <form method="post" action="/nodes/{id}/stop" style="margin-top: 8px;">
                <button type="submit">Stop {name}</button>
            </form>
        </div>"#,
        reason = crate::web::html::escape(reason),
        id = crate::web::html::urlencoding_lite(&node.id),
        name = crate::web::html::escape(&node.name),
    )
}
