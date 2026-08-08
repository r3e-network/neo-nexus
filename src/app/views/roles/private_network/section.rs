/// The three stages of standing up a private network, surfaced one at a time.
///
/// This page used to stack all of them: the template controls, the plan status,
/// the sidecar status, the source-node status, the deployment actions, the signer
/// inputs, and the planned-node grid. Together they laid out ~414pt below a panel
/// that does not scroll, so the Export Launch Pack and Create Nodes buttons and
/// the committee key inputs were all unreachable.
///
/// The split follows the order the work actually happens in: decide the
/// topology, supply the keys that topology needs, then deploy it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum PrivateNetworkSection {
    /// Template, runtime, and the nodes the plan would create.
    Plan,
    /// The committee public keys and their signer references.
    Signers,
    /// Source node, conflicts, and the actions that write the network out.
    Deploy,
}

impl PrivateNetworkSection {
    pub(in crate::app) const ALL: [Self; 3] = [Self::Plan, Self::Signers, Self::Deploy];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Plan => "Plan",
            Self::Signers => "Signers",
            Self::Deploy => "Deploy",
        }
    }

    /// Stable identifier used to persist the active sub-tab across restarts.
    pub(in crate::app) fn persist_key(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Signers => "signers",
            Self::Deploy => "deploy",
        }
    }

    pub(in crate::app) fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|section| section.persist_key() == key)
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/app/views/roles/private_network/section/tests.rs"]
mod tests;
