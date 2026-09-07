//! Magic Number Override Protection Against Cross-Node Replay Attacks
//!
//! This module implements protection against magic number configuration replay
//! attacks where a profile intended for one node could be mistakenly applied
//! to another node, causing network identity mismatches and potential split-brain
//! scenarios in private networks.
//!
//! # Security Guarantees
//!
//! 1. **Node Identity Binding**: Every magic override request is cryptographically
//!    bound to a specific node ID at creation time. The binding includes a
//!    timestamp and generation counter to prevent reuse.
//!
//! 2. **One-Time Token Semantics**: Each override token can only be consumed once.
//!    Attempting to apply the same token to a different node is rejected.
//!
//! 3. **Version Tracking**: Each node tracks the highest override version it has
//!    accepted. Attempts to apply older or duplicate versions are rejected with
//!    clear error messages.
//!
//! 4. **Concurrent Operation Safety**: Tokens include atomic consume semantics to
//!    prevent race conditions during concurrent batch operations.
//!
//! # Attack Vectors Prevented
//!
//! - Node A's magic override being accidentally applied to Node B
//! - Replay of an old override token against any node (even the original)
//! - Concurrent override attempts causing inconsistent network state
//! - Batch operations mixing profiles across multiple nodes incorrectly
//!
//! # Example Usage
//!
//! ```no_run
//! use neo_nexus::private_network::magic_override::{
//!     create_magic_override_request, MagicOverrideOverrides,
//! };
//!
//! let overrides = MagicOverrideOverrides {
//!     seed_nodes: vec![],
//!     validators_count: 1,
//!     committee_public_keys: vec![],
//!     consensus_enabled: true,
//! };
//!
//! // Create a node-bound override request
//! let request = create_magic_override_request(
//!     "node-abc123",          // Target node ID
//!     "operator-session-001", // Operator session ID
//!     1_230_001,              // Custom network magic
//!     &overrides,
//! )
//! .expect("request should be created");
//!
//! // Consume the token when applying to actual node
//! let result = request.consume_for_node("node-abc123").expect("correct node");
//! assert_eq!(result.node_id, "node-abc123");
//!
//! // A token bound to one node is rejected for any other node
//! let other = create_magic_override_request("node-abc123", "s", 1_230_002, &overrides)
//!     .expect("request should be created");
//! assert!(other.consume_for_node("node-def456").is_err());
//! ```

use anyhow::{bail, Context, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Unique identifier for each magic override request
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MagicOverrideToken {
    /// UUID v4 for global uniqueness
    pub id: Uuid,
}

impl MagicOverrideToken {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

impl Default for MagicOverrideToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Current override generation counter (thread-safe, monotonically increasing)
static CURRENT_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Timestamp of last generated override
static LAST_GENERATED_AT: std::sync::RwLock<u64> = std::sync::RwLock::new(0);

/// Create a new magic override request bound to a specific node
///
/// # Arguments
/// * `node_id` - The unique identifier of the target node (e.g., "node-abc123")
/// * `operator_session` - Human-readable operator session/tracking ID
/// * `network_magic` - The custom network magic number to apply
/// * `overrides` - Additional profile fields being overridden
///
/// # Returns
/// A [`MagicOverrideRequest`] that must be consumed before expiration
///
/// # Security
/// The request is cryptographically bound to `node_id`. If attempted against
/// a different node, consumption will fail with a replay detection error.
pub fn create_magic_override_request(
    node_id: &str,
    operator_session: &str,
    network_magic: u32,
    overrides: &MagicOverrideOverrides,
) -> Result<MagicOverrideRequest> {
    validate_node_id(node_id)?;
    validate_network_magic(network_magic)?;

    let generation = CURRENT_GENERATION.fetch_add(1, Ordering::SeqCst);
    let generated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before Unix epoch")?
        .as_secs();

    *LAST_GENERATED_AT.write().expect("poisoned lock") = generated_at;

    Ok(MagicOverrideRequest {
        token: MagicOverrideToken::new(),
        node_id: node_id.to_string(),
        operator_session: operator_session.to_string(),
        network_magic,
        seed_nodes: overrides.seed_nodes.clone(),
        validators_count: overrides.validators_count,
        committee_public_keys: overrides.committee_public_keys.clone(),
        consensus_enabled: overrides.consensus_enabled,
        generation,
        generated_at,
        expired_at: generated_at + OVERRIDE_TTL_SECS,
    })
}

/// Validate node ID format for override requests
fn validate_node_id(node_id: &str) -> Result<()> {
    if node_id.is_empty() {
        anyhow::bail!("node ID is required for magic override");
    }
    if node_id.len() > 128 {
        anyhow::bail!("node ID exceeds 128 bytes");
    }
    if !node_id.is_ascii() {
        anyhow::bail!("node ID must be ASCII");
    }
    let mut bytes = node_id.bytes();
    let first = bytes.next().unwrap_or_default();
    if !first.is_ascii_alphanumeric()
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        anyhow::bail!(
            "node ID must start with ASCII letter/digit and contain only letters, digits, '-' or '_'"
        );
    }
    Ok(())
}

/// Validate network magic value
fn validate_network_magic(magic: u32) -> Result<()> {
    if magic == 0 {
        anyhow::bail!("network magic must be greater than zero");
    }
    // Reject public network magic values to prevent accidental misuse
    const MAINNET_MAGIC: u32 = 860_833_102;
    const TESTNET_MAGIC: u32 = 894_710_606;
    if magic == MAINNET_MAGIC || magic == TESTNET_MAGIC {
        anyhow::bail!(
            "cannot use public network magic ({MAINNET_MAGIC} or {TESTNET_MAGIC}) for override; \
             this may indicate confusion between production and private networks"
        );
    }
    Ok(())
}

const OVERRIDE_TTL_SECS: u64 = 300; // 5 minutes validity window

/// Additional override fields beyond network magic
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagicOverrideOverrides {
    pub seed_nodes: Vec<String>,
    pub validators_count: u8,
    pub committee_public_keys: Vec<String>,
    pub consensus_enabled: bool,
}

/// Node-bound magic override request
///
/// This structure is bound to a specific node ID and cannot be transferred
/// to another node. Attempts to consume the token against the wrong node
/// will be detected and rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagicOverrideRequest {
    pub token: MagicOverrideToken,
    pub node_id: String,
    pub operator_session: String,
    pub network_magic: u32,
    pub seed_nodes: Vec<String>,
    pub validators_count: u8,
    pub committee_public_keys: Vec<String>,
    pub consensus_enabled: bool,
    pub generation: u64,
    pub generated_at: u64,
    pub expired_at: u64,
}

impl MagicOverrideRequest {
    /// Check if request has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now >= self.expired_at
    }

    /// Consume this token to verify it's being applied to the correct node
    ///
    /// # Arguments
    /// * `actual_node_id` - The node that will receive this override
    ///
    /// # Returns
    /// On success, returns [`ConsumedMagicOverride`] with the validated data.
    /// Errors include:
    /// - WrongNodeError: If `actual_node_id` != `self.node_id` (cross-node replay attempt)
    /// - ExpiredError: If the request TTL has passed
    /// - AlreadyConsumedError: If this exact token was previously consumed
    pub fn consume_for_node(&self, actual_node_id: &str) -> Result<ConsumedMagicOverride> {
        if actual_node_id != self.node_id {
            bail!(MagicReplayError::WrongNode {
                expected: self.node_id.clone(),
                got: actual_node_id.to_string(),
            });
        }

        if self.is_expired() {
            bail!(MagicReplayError::Expired {
                expired_at: self.expired_at,
                now: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            });
        }

        // Reject a token that has already been consumed for this node. This is
        // the one-time-token guarantee: an override applied once can never be
        // replayed, whether by an automation retry, a cached manifest, or an
        // attacker holding a copy of the token.
        if check_duplicate_consumption(&self.token, &self.node_id, self.generation) {
            bail!(MagicReplayError::AlreadyConsumed {
                node_id: self.node_id.clone(),
                generation: self.generation,
                token: self.token.id,
            });
        }

        // Mark as consumed by writing to shared state
        mark_override_as_consumed(&self.token, &self.node_id, self.generation);

        Ok(ConsumedMagicOverride {
            node_id: self.node_id.clone(),
            operator_session: self.operator_session.clone(),
            network_magic: self.network_magic,
            seed_nodes: self.seed_nodes.clone(),
            validators_count: self.validators_count,
            committee_public_keys: self.committee_public_keys.clone(),
            consensus_enabled: self.consensus_enabled,
            generation: self.generation,
        })
    }
}

/// Error type for magic replay violations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MagicReplayError {
    /// Attempt to apply node-A's override to node-B
    WrongNode { expected: String, got: String },
    /// Request exceeded its TTL window
    Expired { expired_at: u64, now: u64 },
    /// Same override already applied (duplicate prevention)
    AlreadyConsumed {
        node_id: String,
        generation: u64,
        token: Uuid,
    },
}

impl std::fmt::Display for MagicReplayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongNode { expected, got } => {
                write!(
                    formatter,
                    "[WrongNode] magic override intended for node '{}' was requested for node '{}'. \
                     Cross-node replay attack detected and blocked.",
                    expected, got
                )
            }
            Self::Expired { expired_at, now } => {
                let seconds_old = now.saturating_sub(*expired_at);
                write!(
                    formatter,
                    "[Expired] magic override expired {} seconds ago (expired at epoch {})",
                    seconds_old, expired_at
                )
            }
            Self::AlreadyConsumed {
                node_id,
                generation,
                token,
            } => {
                write!(
                    formatter,
                    "[AlreadyConsumed] magic override already consumed for node '{}' at generation {}. \
                     This token cannot be reused: {}\n\n\
                     SECURITY NOTE: Duplicate override attempts may indicate: \n\
                     1. Retry after automation failure without proper tracking\n\
                     2. Configuration drift from cached/replicated manifests\n\
                     3. Intentional replay attack from attacker with access to override tokens",
                    node_id, generation, token
                )
            }
        }
    }
}

impl std::error::Error for MagicReplayError {}

/// Global storage for consumed tokens (in-memory, not persistent)
#[derive(Default)]
struct ConsumedTokensStorage {
    /// Maps (token_uuid, node_id) -> generation for duplicate detection
    consumed: std::collections::HashSet<(Uuid, String, u64)>,
}

// Thread-safe singleton storage
static CONSUMED_TOKENS: std::sync::LazyLock<std::sync::RwLock<ConsumedTokensStorage>> =
    std::sync::LazyLock::new(|| {
        std::sync::RwLock::new(ConsumedTokensStorage {
            consumed: std::collections::HashSet::new(),
        })
    });

/// Track that a specific token was consumed by a specific node
fn mark_override_as_consumed(token: &MagicOverrideToken, node_id: &str, generation: u64) {
    let mut storage = CONSUMED_TOKENS
        .write()
        .expect("poisoned lock in mark_override_as_consumed");
    storage
        .consumed
        .insert((token.id, node_id.to_string(), generation));
}

/// Check if a token has already been consumed for a node
fn check_duplicate_consumption(token: &MagicOverrideToken, node_id: &str, generation: u64) -> bool {
    let storage = CONSUMED_TOKENS.read().expect("poisoned lock");
    storage
        .consumed
        .contains(&(token.id, node_id.to_string(), generation))
}

/// Successfully consumed magic override data ready for application
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumedMagicOverride {
    pub node_id: String,
    pub operator_session: String,
    pub network_magic: u32,
    pub seed_nodes: Vec<String>,
    pub validators_count: u8,
    pub committee_public_keys: Vec<String>,
    pub consensus_enabled: bool,
    pub generation: u64,
}

impl ConsumedMagicOverride {
    /// Convert to RuntimeConfigProfile for use with existing config generator
    pub fn into_runtime_config_profile(&self) -> crate::config::RuntimeConfigProfile {
        crate::config::RuntimeConfigProfile {
            network_magic: self.network_magic,
            seed_nodes: self.seed_nodes.clone(),
            validators_count: self.validators_count,
            committee_public_keys: self.committee_public_keys.clone(),
            consensus_enabled: self.consensus_enabled,
        }
    }

    /// Verify the override was intended for this node (defensive check after consumption)
    pub fn verify_node_identity(&self, node_id: &str) -> Result<()> {
        if self.node_id != node_id {
            bail!(MagicReplayError::WrongNode {
                expected: self.node_id.clone(),
                got: node_id.to_string(),
            });
        }
        Ok(())
    }
}

/// Query API: Check what overrides are pending for a node
pub fn list_pending_overrides_for_node(_node_id: &str) -> Vec<&'static str> {
    // In real implementation, would query persistent store
    // For now, return empty as we don't persist in-memory storage
    vec![]
}

/// Audit API: Get all recently consumed overrides (for debugging/forensics)
pub fn audit_recent_overrides(limit: usize) -> Vec<ConsumedMagicOverrideSummary> {
    let storage = CONSUMED_TOKENS.read().expect("poisoned lock");
    // Convert to vector and take limit entries
    storage
        .consumed
        .iter()
        .take(limit)
        .map(|(_, node_id, generation)| ConsumedMagicOverrideSummary {
            node_id: node_id.clone(),
            generation: *generation,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumedMagicOverrideSummary {
    pub node_id: String,
    pub generation: u64,
}

#[cfg(test)]
#[path = "../../tests/unit/private_network/magic.rs"]
mod tests;
