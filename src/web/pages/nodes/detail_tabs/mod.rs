//! The tabbed studio views of one node, split by tab.
//!
//! Each tab answers a different question — what this instance is, whether it is
//! healthy, what it is consuming, how it is wired, and who may change it — so
//! each lives in its own file. The re-exports keep the call site in `detail.rs`
//! unchanged.

mod access;
mod configuration;
mod health;
mod monitoring;
mod summary;

pub use access::{render_tab_iac, render_tab_security, render_tab_tags};
pub use configuration::{render_tab_config, render_tab_networking};
pub use health::{render_tab_health, HealthTabFacts};
pub use monitoring::render_tab_monitoring;
pub use summary::{render_tab_details, summary_banner};
