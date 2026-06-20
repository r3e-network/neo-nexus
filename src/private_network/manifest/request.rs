use super::super::*;

pub(in crate::private_network) fn validate_request(
    request: &PrivateNetworkDeploymentRequest,
) -> Result<()> {
    validate_plan_shape(request)?;
    validate_materialized_nodes(request)?;
    validate_committee_capacity(request)
}

fn validate_plan_shape(request: &PrivateNetworkDeploymentRequest) -> Result<()> {
    if request.plan.nodes.is_empty() {
        anyhow::bail!("private network plan is empty");
    }
    if request.plan.consensus_count() == 0 {
        anyhow::bail!("private network plan must include at least one consensus node");
    }
    if request.nodes.len() != request.plan.nodes.len() {
        anyhow::bail!(
            "private network launch pack needs {} nodes but received {}",
            request.plan.nodes.len(),
            request.nodes.len()
        );
    }
    Ok(())
}

fn validate_materialized_nodes(request: &PrivateNetworkDeploymentRequest) -> Result<()> {
    for planned in &request.plan.nodes {
        let node = request
            .nodes
            .iter()
            .find(|node| node.name == planned.name)
            .with_context(|| format!("planned node {} was not materialized", planned.name))?;
        if node.node_type != request.plan.node_type {
            anyhow::bail!(
                "{} runtime {} does not match private network runtime {}",
                node.name,
                node.node_type,
                request.plan.node_type
            );
        }
        if node.network != Network::Private {
            anyhow::bail!("{} is not a private-network node", node.name);
        }
        if matches!(node.status, NodeStatus::Running | NodeStatus::Starting) {
            anyhow::bail!(
                "stop {} before exporting a private network launch pack",
                node.name
            );
        }
    }
    Ok(())
}

fn validate_committee_capacity(request: &PrivateNetworkDeploymentRequest) -> Result<()> {
    let Some(committee) = &request.committee else {
        return Ok(());
    };

    let validators_count = request.plan.consensus_count();
    if committee.signers.len() < validators_count {
        anyhow::bail!(
            "committee signer roster has {} keys but private network requires at least {}",
            committee.signers.len(),
            validators_count
        );
    }
    Ok(())
}
