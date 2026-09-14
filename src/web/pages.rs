//! Page handlers, grouped per workbench destination.

pub mod alerts;
pub mod api_tokens;
pub mod backup;
pub mod config;
pub mod events;
pub mod federation;
pub mod home;
pub mod login;
pub mod logs;
pub mod metrics_page;
pub mod monitor;
pub mod node_editor;
pub mod nodes;
pub mod operations;
pub mod plugins;
pub mod roles;
pub mod runtimes;
pub mod settings;
pub mod signer;
pub mod snapshots;
pub mod wallets;

#[cfg(test)]
#[path = "../../tests/unit/web/honesty/tests.rs"]
mod honesty_tests;

#[cfg(test)]
#[path = "../../tests/unit/web/parity/tests.rs"]
mod parity_tests;

#[cfg(test)]
#[path = "../../tests/unit/web/naming/tests.rs"]
mod naming_tests;
