mod catalog;
mod details;
mod filter;
mod form;
mod layout;
mod registry;
mod section;
mod selection;
mod status;

use eframe::egui;

use crate::app::domain::FastSyncSnapshot;

use super::super::{
    text::short_path,
    theme,
    widgets::{empty_state, metric_row, panel, segmented_control},
    NeoNexusApp, SNAPSHOT_PAGE_SIZE,
};

pub(in crate::app) use section::SnapshotsSection;

use self::status::snapshot_is_verified;
