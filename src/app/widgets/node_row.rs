use eframe::egui;

use crate::app::{
    domain::NodeConfig,
    text::truncate_middle,
    theme::{self, DensityMetrics},
};

use super::badges::{status_badge, status_dot, text_badge};
use super::list_row::list_row_frame;

/// Shared inventory / fleet row: status dot, name, type+network badges, RPC.
/// Uses the frozen v3.1 list selection matrix (accent×0.16, radius 10).
/// Returns `true` when the row was clicked.
pub(in crate::app) fn node_row(
    ui: &mut egui::Ui,
    node: &NodeConfig,
    selected: bool,
    compact: bool,
) -> bool {
    let metrics = DensityMetrics::COMFORTABLE;
    let height = if compact {
        metrics.list_row_compact
    } else {
        metrics.list_row_expanded
    };

    list_row_frame(ui, selected, Some(height), |ui| {
        ui.horizontal(|ui| {
            status_dot(ui, node.status);
            ui.add_space(theme::SM);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        theme::body(truncate_middle(&node.name, if compact { 18 } else { 28 }))
                            .strong(),
                    );
                    if !compact {
                        ui.add_space(theme::SM);
                        status_badge(ui, node.status);
                    }
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    text_badge(ui, &node.node_type.to_string(), theme::muted_text());
                    ui.add_space(theme::XS);
                    text_badge(ui, &node.network.to_string(), theme::info());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(theme::muted_body(format!(":{}", node.rpc_port)));
                        if compact {
                            status_badge(ui, node.status);
                        }
                    });
                });
            });
        });
    })
}
