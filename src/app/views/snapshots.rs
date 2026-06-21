mod catalog;
mod details;
mod filter;
mod form;
mod layout;
mod registry;
mod selection;
mod status;

use eframe::egui;

use crate::app::domain::FastSyncSnapshot;

use super::super::{
    text::short_path,
    widgets::{empty_state, metric_tile, panel},
    NeoNexusApp, SNAPSHOT_PAGE_SIZE,
};

use self::status::snapshot_is_verified;
