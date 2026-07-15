use eframe::egui;

use crate::app::{
    domain::NodeConfig,
    text::truncate_middle,
    theme::{self, DensityMetrics, UiDensity},
};

use super::badges::{status_badge, status_dot, text_badge};
use super::list_row::list_row_frame;

/// Shared inventory / fleet row: selection matrix accent×0.16, radius 10.
///
/// - **Comfortable:** two-line anatomy (name + type/network/RPC).
/// - **Compact:** single-line anatomy (dot + name + badges + port + status)
///   at the denser list heights from [`DensityMetrics::COMPACT`].
///
/// `layout_compact` selects inventory vs fleet slot height within the density.
/// Returns `true` when the row was clicked.
pub(in crate::app) fn node_row(
    ui: &mut egui::Ui,
    node: &NodeConfig,
    selected: bool,
    layout_compact: bool,
    density: UiDensity,
) -> bool {
    let metrics = DensityMetrics::for_density(density);
    let height = if layout_compact {
        metrics.list_row_compact
    } else {
        metrics.list_row_expanded
    };

    list_row_frame(ui, selected, Some(height), |ui| match density {
        UiDensity::Compact => render_single_line(ui, node, layout_compact),
        UiDensity::Comfortable => render_two_line(ui, node, layout_compact),
    })
}

/// Compact single-line: ● name [type] [net] :port [status]
fn render_single_line(ui: &mut egui::Ui, node: &NodeConfig, layout_compact: bool) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = theme::XS;
        status_dot(ui, node.status);
        ui.label(
            theme::body(truncate_middle(
                &node.name,
                if layout_compact { 14 } else { 22 },
            ))
            .strong(),
        );
        text_badge(ui, &node.node_type.to_string(), theme::muted_text());
        text_badge(ui, short_network(node), theme::info());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            status_badge(ui, node.status);
            ui.label(theme::muted_body(format!(":{}", node.rpc_port)));
        });
    });
}

/// Comfortable two-line: name/status on first line, type/network/RPC on second.
fn render_two_line(ui: &mut egui::Ui, node: &NodeConfig, layout_compact: bool) {
    ui.horizontal(|ui| {
        status_dot(ui, node.status);
        ui.add_space(theme::SM);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    theme::body(truncate_middle(
                        &node.name,
                        if layout_compact { 18 } else { 28 },
                    ))
                    .strong(),
                );
                if !layout_compact {
                    ui.add_space(theme::SM);
                    status_badge(ui, node.status);
                }
            });
            ui.add_space(theme::XS);
            ui.horizontal(|ui| {
                text_badge(ui, &node.node_type.to_string(), theme::muted_text());
                ui.add_space(theme::XS);
                text_badge(ui, &node.network.to_string(), theme::info());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(theme::muted_body(format!(":{}", node.rpc_port)));
                    if layout_compact {
                        status_badge(ui, node.status);
                    }
                });
            });
        });
    });
}

fn short_network(node: &NodeConfig) -> &'static str {
    // Keep Compact single-line badges short so 40pt rows do not wrap.
    use crate::app::domain::Network;
    match node.network {
        Network::Mainnet => "main",
        Network::Testnet => "test",
        Network::Private => "priv",
    }
}
