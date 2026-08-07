//! Read-only views of chain state that a node's duties depend on.
//!
//! Some of what an operator needs to know about a node is not in its
//! configuration at all: whether the committee has designated its key for the
//! Oracle role, who the current committee is, how the candidate vote stands.
//! Those live on chain, and this module reads them over ordinary JSON-RPC.
//!
//! **It reads. It never signs.** Designating a role is a committee-witnessed
//! transaction and registering a candidate costs GAS; both need a private key
//! NeoNexus does not hold and should not ask for. A manager that offered a
//! "Designate" button it cannot honour would be worse than one that reports
//! "not designated" and says who can change that.

mod designation;
mod governance;
mod model;
mod rpc;

pub use designation::designation_status;
pub use governance::governance_snapshot;
pub use model::{
    CandidateStanding, ChainQueryError, DesignationStatus, GovernanceSnapshot, RoleDesignation,
};
