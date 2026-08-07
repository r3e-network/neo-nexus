//! What the generator needs to know about a node beyond its definition.
//!
//! A node's *duty* decides which service sections its configuration carries,
//! and its wallet decides whether those services can be switched on. Neither
//! lives on `NodeConfig`: the duty is recorded against the node in the
//! repository, and the wallet is an operator secret supplied per export. They
//! travel together here so a generator signature does not grow a parameter
//! every time a duty needs one more fact.

use crate::roles::NodeRole;

/// A wallet a signing service unlocks at startup. The password is written to
/// the config file in plaintext — that is how both clients read it — so this is
/// only ever populated when an operator has explicitly supplied one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceWallet {
    pub path: String,
    pub password: String,
}

/// Extra inputs for a config render. The default — no duty, no wallet — renders
/// exactly what NeoNexus rendered before duties existed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GenerationContext {
    pub role: Option<NodeRole>,
    pub wallet: Option<ServiceWallet>,
}

impl GenerationContext {
    pub fn for_role(role: NodeRole) -> Self {
        Self {
            role: Some(role),
            wallet: None,
        }
    }

    pub fn with_wallet(mut self, wallet: ServiceWallet) -> Self {
        self.wallet = Some(wallet);
        self
    }
}
