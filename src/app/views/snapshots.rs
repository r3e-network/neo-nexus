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

use crate::app::domain::{list_workspace_snapshots, FastSyncSnapshot};

use super::super::{
    text::{short_path, truncate_middle},
    theme,
    widgets::{empty_state, metric_row, page_chrome, panel},
    NeoNexusApp, SNAPSHOT_PAGE_SIZE,
};

pub(in crate::app) use section::SnapshotsSection;

use self::status::snapshot_is_verified;
