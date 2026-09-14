//! Whether a rendered Neo N3 config describes a node that can actually join a
//! chain.
//!
//! Every existing check compares the file against **what this workspace would
//! generate**. That catches tampering and drift, and it is structurally unable
//! to catch the case where what the workspace generates is itself unusable:
//! `effective_seed_nodes(Private, None)` is `[]`, and `len >= 0` passes.
//!
//! Reproduced with `--generate-node-config` before this was written:
//!
//! * neo-go, private → `SeedList: []`, `StandbyCommittee: []`, and the report
//!   said **"ready, 10 pass, 0 critical"**.
//! * neo-rs, private → `seed_nodes = []`, "ready, 24 pass".
//! * neo-cli, private → the entire `ProtocolConfiguration` was
//!   `{"Network": 1230000}`. That one is the worst: the generator only writes
//!   `SeedList`/`ValidatorsCount`/`StandbyCommittee` when a profile is present,
//!   so neo-cli falls back to its **compiled-in public defaults** and dials
//!   MainNet seeds while carrying a private network magic.
//!
//! The Neo X path already fails closed on the equivalent gap. These checks give
//! the N3 path the same property: a node that cannot work is never called ready.

use crate::types::{Network, NodeConfig};

use super::super::model::ConfigValidationReport;

/// What a generated config actually contains, independent of format.
///
/// The three N3 clients disagree about spelling and nesting, so each validator
/// extracts these and hands them here — one rule, three parsers, rather than
/// three rules that drift.
pub(in crate::config::validation) struct ChainIdentity {
    /// `None` when the key is absent entirely, which for neo-cli means the
    /// client silently uses its own compiled-in list.
    pub seed_count: Option<usize>,
    pub committee_count: Option<usize>,
    pub validators_count: Option<u64>,
    /// Whether this client's config format can carry a committee at all.
    ///
    /// neo-rs takes none: its config has no such key and the client holds its
    /// own. An absent committee there is the schema, not an omission, and
    /// reporting it as one would be a finding an operator can never clear.
    pub committee_is_expressible: bool,
}

/// Check that the node has somewhere to get blocks from and someone to trust.
pub(in crate::config::validation) fn check_chain_identity(
    report: &mut ConfigValidationReport,
    node: &NodeConfig,
    identity: &ChainIdentity,
) {
    check_seeds(report, node, identity.seed_count);
    check_committee(report, node, identity);
}

/// A node with no seeds and no peers has nowhere to get blocks from.
///
/// The severity turns on *which* failure it is, because they are not the same
/// problem and an operator acts on them differently:
///
/// * **Absent** keys are critical. The client falls back to what it was
///   compiled with — the public seeds — so a node carrying a private magic
///   actively dials MainNet. It joins the wrong network rather than no network.
/// * **Present but empty** is a warning. The node will not sync, which is
///   serious, but it is also exactly right for a single-node private chain that
///   produces its own blocks. Refusing to write that config would strand a
///   legitimate setup, so it is reported and allowed.
///
/// The distinction matters because a critical finding stops the config being
/// written at all.
fn check_seeds(report: &mut ConfigValidationReport, node: &NodeConfig, seed_count: Option<usize>) {
    match seed_count {
        Some(count) if count > 0 => report.pass(
            "Seed list",
            format!("{count} seed node(s) to synchronise from."),
        ),
        Some(_) if node.network == Network::Private => report.warning(
            "Seed list",
            "The seed list is empty, so this node has nowhere to get blocks from. That is \
             correct only for a single node that produces its own blocks — which also needs a \
             consensus duty and its own key. A private network with other members needs their \
             addresses here, or this node will sit at height 0.",
        ),
        Some(_) => report.critical(
            "Seed list",
            format!(
                "The seed list is empty, so this node cannot reach {}. Public networks have \
                 published seeds and this config carries none.",
                node.network
            ),
        ),
        None => report.critical(
            "Seed list",
            format!(
                "No seed list is present in the config at all, so {} will fall back to the \
                 addresses compiled into the client — which are the public ones, whatever \
                 network this node is configured for.",
                node.node_type
            ),
        ),
    }
}

/// A committee is who the node believes may produce blocks.
///
/// An absent committee on a private network is the shape of G18's worst case: a
/// node carrying a private magic and a public committee cannot validate a
/// single block its own network produces, and will not say so — it simply never
/// accepts anything. Critical, because it is wrong rather than incomplete.
///
/// An *empty* committee is a warning for the same reason an empty seed list is:
/// serious, and legitimate for a single-node chain whose own key is the whole
/// committee, supplied at launch rather than in the file.
fn check_committee(
    report: &mut ConfigValidationReport,
    node: &NodeConfig,
    identity: &ChainIdentity,
) {
    let validators = identity.validators_count;
    match identity.committee_count {
        Some(0) => report.warning(
            "Standby committee",
            "The standby committee is empty, so this node trusts nobody to produce blocks and \
             will reject every block it receives — unless its own consensus key is supplied at \
             launch and it is the whole committee.",
        ),
        Some(count) => {
            match validators {
                Some(validators) if validators as usize > count => report.critical(
                    "Standby committee",
                    format!(
                        "ValidatorsCount is {validators} but the committee holds {count} key(s). \
                         The node cannot elect more validators than it has committee members, \
                         and will refuse to start."
                    ),
                ),
                _ => report.pass(
                    "Standby committee",
                    format!("{count} committee key(s) configured."),
                ),
            };
        }
        None if !identity.committee_is_expressible => report.pass(
            "Standby committee",
            format!(
                "{} takes no committee in its config; the client holds its own.",
                node.node_type
            ),
        ),
        None if node.network == Network::Private => report.critical(
            "Standby committee",
            format!(
                "No standby committee is present in the config, so {} will use the committee \
                 compiled into it — the public one — while this node carries a private network \
                 magic. It will reject every block its own network produces.",
                node.node_type
            ),
        ),
        // On a public network the client's compiled-in committee *is* the
        // right one, so this is a pass rather than a warning. A warning that
        // fires on every correctly-configured public node teaches operators to
        // ignore warnings, which costs more than the hardening it asks for.
        None => report.pass(
            "Standby committee",
            format!(
                "Not pinned; {} uses its own committee for {}, which is the published one.",
                node.node_type, node.network
            ),
        ),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/config/validation/chain_identity/tests.rs"]
mod tests;
