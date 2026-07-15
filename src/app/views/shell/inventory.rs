use eframe::egui;

use crate::app::domain::NodeStatus;

use super::super::super::{
    paging::page_count,
    theme,
    widgets::{
        chip_pill, empty_state, empty_state_with_action, filter_bar, filter_chip, mini_stat,
        node_row, pagination_bar,
    },
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
