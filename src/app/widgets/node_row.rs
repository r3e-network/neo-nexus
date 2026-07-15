use eframe::egui;

use crate::app::{
    domain::NodeConfig,
    text::truncate_middle,
    theme::{self, card_surface},
};

use super::badges::{status_badge, status_dot, text_badge};

/// Shared inventory / fleet row: status dot, name, type+network badges, RPC.
/// Returns `true` when the row was clicked.
pub(in crate::app) fn node_row(
    ui: &mut egui::Ui,
    node: &NodeConfig,
    selected: bool,
    compact: bool,
) -> bool {
    let width = ui.available_width();
    let height = if compact { 44.0 } else { 56.0 };
    let fill = if selected {
        theme::accent().gamma_multiply(0.18)
    } else {
        card_surface()
    };
    let stroke = if selected {
        egui::Stroke::new(1.0, theme::accent())
    } else {
        theme::hairline()
    };

    let mut clicked = false;
    let response = egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_min_size(egui::vec2(width - 4.0, height - 8.0));
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
                        text_badge(
                            ui,
                            &node.node_type.to_string(),
                            theme::muted_text(),
                        );
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
        .response
        .interact(egui::Sense::click());

    if response.clicked() {
        clicked = true;
    }
    if response.hovered() && !selected {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    clicked
}
