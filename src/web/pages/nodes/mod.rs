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
