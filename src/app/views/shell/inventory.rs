use eframe::egui;

use crate::app::domain::NodeStatus;

use super::super::super::{
    paging::page_count,
    text::truncate_middle,
    theme::{muted_text, status_color},
    widgets::{empty_state, fact, pagination_bar},
    NeoNexusApp, NODE_PAGE_SIZE,
};

mod filter;
mod header;
mod list;
mod metrics;

impl NeoNexusApp {
    pub(in crate::app) fn render_inventory_panel(&mut self, ui: &mut egui::Ui) {
        self.render_inventory_header(ui);
        self.render_inventory_metrics(ui);
        self.render_inventory_filter(ui);
        ui.separator();
        self.render_inventory_nodes(ui);
    }
}
