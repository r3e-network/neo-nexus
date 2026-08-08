use super::super::*;

pub(in crate::private_network) fn deployment_network_magic(
    template: PrivateNetworkTemplate,
    node_type: NodeType,
) -> u32 {
    let runtime_offset = match node_type {
        NodeType::NeoCli => 100,
        NodeType::NeoGo => 200,
        NodeType::NeoRs => 300,
        // Neo X private networks are not planned by this template: they need a
        // genesis allocation, not a committee roster. The offsets exist so the
        // match stays exhaustive.
        NodeType::NeoXGeth => 400,
        NodeType::NeoXReth => 500,
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
