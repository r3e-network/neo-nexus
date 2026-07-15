use eframe::egui;

use crate::app::domain::{NodeRole, RolePlanner};

use super::super::super::{
    text::truncate_middle,
    theme::{self, muted_text, status_color},
    widgets::{empty_state, fact, grid_header, plugin_enabled},
    NeoNexusApp,
};

mod presets;
mod selected;
