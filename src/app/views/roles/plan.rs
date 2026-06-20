use eframe::egui;

use crate::{
    roles::{NodeRole, RolePlanner},
    types::NodeStatus,
};

use super::super::super::{
    text::truncate_middle,
    theme::{muted_text, status_color},
    widgets::{empty_state, fact, plugin_enabled},
    NeoNexusApp,
};

mod presets;
mod selected;
