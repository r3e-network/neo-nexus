//! The duty a node is recorded as performing, read back for a launch.
//!
//! A node's duty and its wallet live in the workspace, not in the config file.
//! Every render therefore has to look them up, and the answer has to be the same
//! whichever surface asks — the workbench's Apply Config, the CLI's export, and
//! the launch itself.
//!
//! It was not. Launch wrote the managed config with no context at all, so
//! pressing Start on a node whose duty had just been applied overwrote its
//! `Consensus:` / `Oracle:` section with a plain relaying config and started it
//! as a relay. The workbench went on showing the duty, because the duty *was*
//! recorded — it simply never reached the file the node booted from.
//!
//! The wallet arrives by path only. Enabling a signing service writes its
//! password into the config in plaintext, and NeoNexus never holds that
//! password, so the section is written with an empty one and stays disabled
//! until an operator supplies it.

use crate::{
    config::{GenerationContext, ServiceWallet},
    repository::Repository,
    types::NodeConfig,
};

/// Reads a node's recorded duty and wallet out of the workspace.
///
/// A read failure yields no duty rather than an error: a node that cannot be
/// told what it is supposed to do must fall back to relaying, which is inert,
/// not to a signing role it might not be configured for.
pub fn generation_context_for_node(
    repository: &Repository,
    node: &NodeConfig,
) -> GenerationContext {
    GenerationContext {
        role: repository.load_node_role(&node.id).unwrap_or_default(),
        wallet: service_wallet_for_node(repository, node),
    }
}

fn service_wallet_for_node(repository: &Repository, node: &NodeConfig) -> Option<ServiceWallet> {
    let profile_id = repository.load_node_wallet(&node.id).ok().flatten()?;
    repository
        .list_neo_wallet_profiles()
        .ok()?
        .into_iter()
        .find(|profile| profile.id == profile_id)
        .map(|profile| ServiceWallet::at(profile.source_path))
}
