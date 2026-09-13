//! One node, one round of sampling, and what it produced.

use crate::observe::evidence::{NotSampled, Observation};

/// Everything one round learned about one node.
///
/// Every field is an [`Observation`], so a round that read the height but could
/// not read the peer count is representable — and renders as exactly that,
/// rather than as a peer count of zero. That distinction is not academic: zero
/// peers is [`crate::observe::HealthState::Isolated`], a state that pages
/// someone, and a node whose client does not implement `getconnectioncount`
/// must never enter it.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeSample {
    pub node_id: String,
    pub sampled_at_unix: u64,
    /// The endpoint these answers came from, recorded because it changes when
    /// an operator edits the RPC port and old rows should not look current.
    pub endpoint: String,

    /// Blocks the node holds, normalised across families so one operator-facing
    /// number means the same thing on Neo N3 and Neo X.
    pub block_height: Observation<u64>,
    /// Headers the node holds. On Neo N3 a node fetches headers ahead of
    /// blocks, so `header_height - block_height` is how far behind its own
    /// knowledge it is — a sync signal available with no reference node.
    pub header_height: Observation<u64>,
    /// Neo X reports sync state directly; `false` means caught up.
    pub syncing: Observation<bool>,

    /// When the newest block was produced, in seconds.
    ///
    /// Neo N3 reports this in **milliseconds** and Neo X in seconds. The
    /// normalisation happens at the parser and is tested, because a missed
    /// division makes every Neo N3 node read as roughly fifty-four thousand
    /// years behind.
    pub head_block_time_unix: Observation<u64>,

    pub peers_connected: Observation<u32>,

    /// The network magic or chain id the node actually joined — not the one it
    /// was configured with. The two differing is the whole of G18.
    pub observed_magic: Observation<u64>,
    /// Milliseconds between blocks, as the node reports them. Every stall and
    /// sync threshold derives from this rather than from a constant.
    pub ms_per_block: Observation<u64>,
    pub mempool_capacity: Observation<u64>,
    pub validators_count: Observation<u32>,
    pub client_version: Observation<String>,

    pub mempool_verified: Observation<u64>,
    pub mempool_unverified: Observation<u64>,

    /// Round-trip time of the head call, the one method sampled every round —
    /// so this figure means one thing rather than an average over a changing
    /// set of calls.
    pub head_latency_ms: Option<u32>,
    /// Whether the node answered the liveness class at all.
    pub head_ok: bool,
}

impl NodeSample {
    /// A round in which nothing was asked, for a node that cannot be.
    ///
    /// Used where `rpc_port == 0`: there is no endpoint to call, and the
    /// absence has to say so rather than look like a node that failed.
    pub fn not_observable(node_id: impl Into<String>, sampled_at_unix: u64) -> Self {
        Self::empty(node_id, sampled_at_unix, String::new(), || {
            NotSampled::SamplingDisabled
        })
    }

    /// A round against a node that did not answer its liveness call.
    pub fn unreachable(
        node_id: impl Into<String>,
        sampled_at_unix: u64,
        endpoint: impl Into<String>,
        reason: NotSampled,
    ) -> Self {
        Self::empty(node_id, sampled_at_unix, endpoint, || reason.clone())
    }

    fn empty(
        node_id: impl Into<String>,
        sampled_at_unix: u64,
        endpoint: impl Into<String>,
        reason: impl Fn() -> NotSampled,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            sampled_at_unix,
            endpoint: endpoint.into(),
            block_height: Observation::Unknown(reason()),
            header_height: Observation::Unknown(reason()),
            syncing: Observation::Unknown(reason()),
            head_block_time_unix: Observation::Unknown(reason()),
            peers_connected: Observation::Unknown(reason()),
            observed_magic: Observation::Unknown(reason()),
            ms_per_block: Observation::Unknown(reason()),
            mempool_capacity: Observation::Unknown(reason()),
            validators_count: Observation::Unknown(reason()),
            client_version: Observation::Unknown(reason()),
            mempool_verified: Observation::Unknown(reason()),
            mempool_unverified: Observation::Unknown(reason()),
            head_latency_ms: None,
            head_ok: false,
        }
    }

    /// The key identifying the chain this node actually joined.
    ///
    /// Sampling groups by this rather than by the configured `Network` enum,
    /// which is what lets head lag be computed before a `networks` table
    /// exists — and what stops a node on magic 1230000 being compared against
    /// one on MainNet because both rows say "private".
    pub fn chain_key(&self, family: crate::types::ChainFamily) -> Option<String> {
        self.observed_magic
            .value()
            .map(|magic| format!("{family}:{magic}"))
    }
}

/// One sampling pass over the fleet.
#[derive(Clone, Debug, Default)]
pub struct SampleRound {
    pub samples: Vec<NodeSample>,
}

impl SampleRound {
    pub fn get(&self, node_id: &str) -> Option<&NodeSample> {
        self.samples.iter().find(|sample| sample.node_id == node_id)
    }
}
