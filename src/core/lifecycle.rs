//! Node lifecycle orchestration: the shared launch/restart pipeline usable by the
//! GUI shell and a headless CLI. See [`crate::node_lifecycle`] for the
//! implementation; this re-export exposes it through the core facade so both
//! frontends import it as `crate::core::node_lifecycle`.
pub use crate::node_lifecycle::*;
