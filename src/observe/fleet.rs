//! What a node can be compared against.
//!
//! Pure, like the rest of the derivations, so the loop that judges the fleet and
//! the pages that render it resolve reference heads the same way rather than
//! each having an opinion.

use std::collections::BTreeMap;

use super::{derive::ReferenceHead, sample::NodeSample};
use crate::types::NodeConfig;

/// Where each node's chain believes its head to be.
///
/// Grouped by the magic a node **actually joined**, read from its own
/// `getversion`, not by the `Network` it was configured with. A node set to a
/// private network that fell back to compiled-in MainNet defaults is on
/// MainNet, and comparing its height against the other private nodes' would
/// report a lag of several hundred million blocks instead of the configuration
/// error that caused it.
///
/// A node absent from the result has no reference and is not compared. That
/// covers a node whose magic has not been read yet and — deliberately — a node
/// alone on its chain: comparing a node against itself always puts it exactly
/// at the head, and "0 blocks behind" on a single-node private chain is a
/// number that means nothing while reading as reassurance.
pub fn reference_heads(
    nodes: &[NodeConfig],
    histories: &BTreeMap<String, Vec<NodeSample>>,
) -> BTreeMap<String, ReferenceHead> {
    let mut chains: BTreeMap<String, Vec<(&NodeConfig, u64)>> = BTreeMap::new();
    for node in nodes {
        let Some(latest) = histories.get(&node.id).and_then(|history| history.first()) else {
            continue;
        };
        let (Some(key), Some(height)) = (
            latest.chain_key(node.node_type.family()),
            latest.block_height.value().copied(),
        ) else {
            continue;
        };
        chains.entry(key).or_default().push((node, height));
    }

    let mut heads = BTreeMap::new();
    for members in chains.into_values() {
        if members.len() < 2 {
            continue;
        }
        let Some((holder, height)) = members.iter().max_by_key(|(_, height)| *height) else {
            continue;
        };
        for (node, _) in &members {
            heads.insert(
                node.id.clone(),
                ReferenceHead::Known {
                    height: *height,
                    source: holder.name.clone(),
                },
            );
        }
    }
    heads
}
