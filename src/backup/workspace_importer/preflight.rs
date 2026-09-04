use crate::{backup::schema::WorkspaceBackup, repository::Repository, types::NodeStatus};
use anyhow::{bail, Result};

/// Run again inside the import transaction; the public preview is advisory.
pub(super) fn validate_target(repository: &Repository, backup: &WorkspaceBackup) -> Result<()> {
    let nodes = repository.list_nodes()?;
    let agents = repository.list_agents()?;
    let recoveries = repository.load_node_recoveries()?;
    for incoming in &backup.nodes {
        if recoveries
            .get(&incoming.id)
            .is_some_and(|state| state.next_attempt_at_unix_ms.is_some() || state.claim.is_some())
        {
            bail!(
                "stop node {} to cancel its pending automatic recovery before restoring",
                incoming.id
            );
        }
        if nodes.iter().any(|node| {
            node.id == incoming.id
                && (node.pid.is_some()
                    || matches!(node.status, NodeStatus::Running | NodeStatus::Starting))
        }) {
            bail!(
                "stop node {} before restoring its configuration; its process state was preserved",
                incoming.id
            );
        }
        if agents.iter().any(|agent| {
            agent.profile.node_id.as_deref() == Some(&incoming.id)
                && (agent.pid.is_some()
                    || agent.desired_running
                    || matches!(agent.status, crate::agents::AgentStatus::Running))
        }) {
            bail!(
                "stop the agent associated with node {} before restoring that node",
                incoming.id
            );
        }
    }
    // An ID is a stable signing identity, not permission to replace keys or
    // local file references already used by unrelated nodes in this workspace.
    for existing in repository.list_neo_wallet_profiles()? {
        if let Some(incoming) = backup
            .neo_wallet_profiles
            .iter()
            .find(|profile| profile.id == existing.id)
        {
            if existing.source_path != incoming.source_path
                || existing.wallet_sha256 != incoming.wallet_sha256
                || existing.primary_address != incoming.primary_address
                || existing.contract_public_keys != incoming.contract_public_keys
            {
                bail!("wallet profile {} conflicts with the local signing identity; review it separately or restore into a new workspace", existing.id);
            }
        }
    }
    for existing in repository.list_runtime_signer_profiles()? {
        if backup.runtime_signer_profiles.iter().any(|incoming| {
            incoming.id == existing.id && incoming.ed25519_public_key != existing.ed25519_public_key
        }) {
            bail!("runtime signer profile {} conflicts with a local trusted key; review it separately or restore into a new workspace", existing.id);
        }
    }
    for existing in repository.list_runtime_catalog_profiles()? {
        if backup.runtime_catalog_profiles.iter().any(|incoming| {
            incoming.id == existing.id
                && (incoming.source != existing.source
                    || incoming.signature_source != existing.signature_source
                    || incoming.ed25519_public_key != existing.ed25519_public_key)
        }) {
            bail!("runtime catalog profile {} conflicts with a local source or trusted key; review it separately or restore into a new workspace", existing.id);
        }
    }
    Ok(())
}
