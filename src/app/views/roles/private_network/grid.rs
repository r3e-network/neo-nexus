use eframe::egui;

use crate::app::domain::{PrivateNetworkNodePlan, PrivateNetworkPlan};

use super::super::super::super::paging::rows_that_fit;
use super::super::super::super::text::truncate_middle;
use super::super::super::super::theme::muted_text;
use super::super::super::super::widgets::grid_header;

/// The largest plan any template produces, and so the most rows worth reserving.
const PRIVATE_PLAN_ROWS: usize = 7;
const PRIVATE_PLAN_COLUMNS: usize = 7;

/// One grid row's real cost, measured rather than assumed.
///
/// A striped `egui::Grid` row at the app's 13pt body font is taller than the
/// text: it carries the row's interact height, the stripe padding and
/// `item_spacing.y`. Estimating it at 22pt left seven rows 46pt past the panel;
/// this value was found by sweeping until the containment contract went green,
/// and undercounting it is silently expensive because the error multiplies by
/// the row count.
const PLAN_ROW_HEIGHT: f32 = 32.0;

/// The header row, the note line under the grid, and the separator above it.
const PLAN_CHROME_HEIGHT: f32 = 58.0;

/// The planned nodes, as many as the panel can hold.
///
/// This used to render a fixed seven rows whatever the height, which pushed the
/// last of them ~130pt below a panel that does not scroll. A row an operator
/// cannot see is worse than one that is counted: egui culls it entirely, so the
/// grid looked complete while hiding planned nodes. Now the row count follows
/// the available height and anything left over is stated rather than dropped.
pub(super) fn render_plan_grid(ui: &mut egui::Ui, plan: &PrivateNetworkPlan) {
    let visible = rows_that_fit(ui.available_height(), PLAN_ROW_HEIGHT, PLAN_CHROME_HEIGHT)
        .min(PRIVATE_PLAN_ROWS);

    egui::Grid::new("private_network_plan")
        .striped(true)
        .min_col_width(74.0)
        .show(ui, |ui| {
            render_header(ui);
            for row in 0..visible {
                render_row(ui, plan.nodes.get(row));
                ui.end_row();
            }
        });

    let hidden = plan.nodes.len().saturating_sub(visible);
    if hidden > 0 {
        ui.label(
            egui::RichText::new(format!(
                "{hidden} more planned node(s) not shown — widen the window or hide the inspector."
            ))
            .color(muted_text()),
        );
    }
}

fn render_header(ui: &mut egui::Ui) {
    grid_header(
        ui,
        &["Name", "Runtime", "Role", "RPC", "P2P", "WS", "Storage"],
    );
}

fn render_row(ui: &mut egui::Ui, node: Option<&PrivateNetworkNodePlan>) {
    if let Some(node) = node {
        ui.label(truncate_middle(&node.name, 26));
        ui.label(node.node_type.to_string());
        ui.label(node.role.label());
        ui.label(node.rpc_port.to_string());
        ui.label(node.p2p_port.to_string());
        ui.label(
            node.ws_port
                .map_or_else(|| "-".to_string(), |port| port.to_string()),
        );
        ui.label(node.node_type.storage_label(node.storage_engine));
    } else {
        for _ in 0..PRIVATE_PLAN_COLUMNS {
            ui.label(" ");
        }
    }
}
