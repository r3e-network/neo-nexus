use super::super::*;

pub(in crate::private_network) fn deployment_network_magic(
    template: PrivateNetworkTemplate,
    node_type: NodeType,
) -> u32 {
    let runtime_offset = match node_type {
        NodeType::NeoCli => 100,
        NodeType::NeoGo => 200,
        NodeType::NeoRs => 300,
    };
    let template_offset = match template {
        PrivateNetworkTemplate::SingleValidator => 1,
        PrivateNetworkTemplate::FourValidators => 4,
        PrivateNetworkTemplate::SevenNodeLab => 7,
    };
    1_230_000 + runtime_offset + template_offset
}

pub(in crate::private_network) fn seed_nodes(plan: &PrivateNetworkPlan) -> Vec<String> {
    plan.nodes
        .iter()
        .filter(|node| node.role == NodeRole::Consensus)
        .map(|node| format!("127.0.0.1:{}", node.p2p_port))
        .collect()
}
