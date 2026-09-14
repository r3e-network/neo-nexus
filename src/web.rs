//! The web workbench: a browser-accessible operations surface served by the
//! same binary. It is the third frontend of the core facade — the GUI was the
//! first and the CLI the second. Pages read through the repository and the
//! `core::` facade and add no business logic of their own, so a browser
//! operator and a script operator reach the same decisions by the same code.
//!
//! The sidebar destinations live in [`nav`], which the end-to-end suite walks.
//! Pages render server-side from Rust string templates (the source-purity gate
//! keeps JS/CSS/HTML out of the tree as standalone files), and the JSON API
//! powers light polling from the embedded script.

pub mod api;
pub mod api_tokens;
pub mod assets;
pub mod auth;
pub mod chain_state_view;
pub mod control;
pub mod fleet;
pub mod health;
pub mod html;
pub mod jobs;
pub mod nav;
pub mod node_form;
pub mod pages;
pub mod plugin_ops;
pub mod public_api;
pub mod router;
pub mod runtime_ops;
mod security_headers;
pub mod server;
pub mod signer_api;
pub mod signer_control;
pub mod snapshot_ops;
pub mod state;
pub mod time;
pub mod wallet_ops;

pub use server::{run_web_server, WebLaunch};
pub use state::{Admin, Custody, WebState};
