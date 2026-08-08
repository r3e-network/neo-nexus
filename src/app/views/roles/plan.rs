use eframe::egui;

use crate::app::domain::{NodeRole, NodeStatus, NodeType, RolePlanner};

use super::super::super::{
    text::truncate_middle,
    theme::{self, muted_text, status_color},
    widgets::{empty_state, fact, fits_side_by_side_at, grid_header, hr_tight, plugin_enabled},
    NeoNexusApp,
};

mod presets;
mod selected;
