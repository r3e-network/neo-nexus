use crate::signing::SignerBackendKind;

/// Capabilities are data, not guesses made by individual pages or callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignerCapabilities {
    pub neo_n3_transaction: bool,
    pub neo_n3_consensus: bool,
    pub neo_n3_raw: bool,
    pub neox_transaction: bool,
    pub neox_eip191: bool,
    pub key_administration: bool,
    pub policy_administration: bool,
    pub caller_administration: bool,
    pub durable_audit: bool,
    pub public_relay: bool,
}

impl SignerCapabilities {
    pub fn service() -> Self {
        Self {
            neo_n3_transaction: true,
            neo_n3_consensus: true,
            neo_n3_raw: true,
            neox_transaction: true,
            neox_eip191: true,
            key_administration: true,
            policy_administration: true,
            caller_administration: true,
            durable_audit: true,
            public_relay: true,
        }
    }

    /// A local `secure-sign-service-rs` deployment is a consensus-only native
    /// node signer.  It has no NeoOS key, caller, policy, audit, transaction,
    /// or public-relay HTTP surface.
    pub fn local_signer() -> Self {
        Self {
            neo_n3_transaction: false,
            neo_n3_consensus: true,
            neo_n3_raw: false,
            neox_transaction: false,
            neox_eip191: false,
            key_administration: false,
            policy_administration: false,
            caller_administration: false,
            durable_audit: false,
            public_relay: false,
        }
    }

    pub fn local_wallet(transaction: bool, _consensus: bool, raw: bool) -> Self {
        Self {
            neo_n3_transaction: transaction,
            // A local encrypted wallet has no durable anti-equivocation state.
            // Do not advertise consensus even if a stale embedding config sets
            // the legacy field.
            neo_n3_consensus: false,
            neo_n3_raw: raw,
            neox_transaction: false,
            neox_eip191: false,
            key_administration: false,
            policy_administration: false,
            caller_administration: false,
            durable_audit: false,
            public_relay: false,
        }
    }

    pub fn for_kind(kind: SignerBackendKind) -> Self {
        match kind {
            SignerBackendKind::LocalWallet => Self::local_wallet(true, false, false),
            SignerBackendKind::LocalSigner => Self::local_signer(),
            SignerBackendKind::NeoOsService => Self::service(),
        }
    }
}
