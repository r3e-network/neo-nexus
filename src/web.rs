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
pub mod assets;
pub mod auth;
pub mod control;
pub mod fleet;
pub mod health;
pub mod html;
pub mod nav;
pub mod pages;
pub mod router;
pub mod server;
pub mod state;

pub use server::{run_web_server, WebLaunch};
pub use state::WebState;
