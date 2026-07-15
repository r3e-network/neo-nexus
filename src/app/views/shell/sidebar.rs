use eframe::egui;

use super::super::super::{
    shortcuts::{labels::shortcut_hint, AppShortcut},
    theme,
    view::View,
    NeoNexusApp,
};

// v3 primary navigation: six destinations.
const NAV_GROUPS: &[(&str, &[View])] = &[
    ("Workspace", &[View::Summary, View::Operations]),
    ("Nodes", &[View::Nodes, View::Runtimes]),
    ("Network", &[View::Federation]),
    ("System", &[View::Settings]),
];

impl NeoNexusApp {
    pub(in crate::app) fn render_navigation_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(theme::XS);
        ui.horizontal(|ui| {
            ui.label(theme::metric_value(theme::brand_glyph()).color(theme::accent()));
            ui.add_space(theme::XS);
            ui.vertical(|ui| {
                ui.label(theme::page_title("NeoNexus"));
                ui.label(theme::muted_body("Neo node operations"));
            });
        });
        ui.add_space(theme::SM);
        ui.separator();
        ui.add_space(theme::SM);

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(theme::XS);
            self.render_sidebar_controls(ui);
            ui.add_space(theme::SM);
            ui.separator();
            ui.add_space(theme::SM);

            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                self.render_navigation_items(ui);
            });
        });
    }

    fn render_navigation_items(&mut self, ui: &mut egui::Ui) {
        for (index, (group, views)) in NAV_GROUPS.iter().enumerate() {
            if index > 0 {
                ui.add_space(theme::MD);
            }
            ui.label(theme::label_caption(*group));
            ui.add_space(theme::XS);
            for &view in *views {
                self.render_nav_item(ui, view);
                ui.add_space(2.0);
            }
        }
    }

    fn render_nav_item(&mut self, ui: &mut egui::Ui, view: View) {
        let selected = self.session.selected_view.primary_nav() == view;
        let width = ui.available_width();
        let icon = theme::view_icon_glyph(view);
        let label = format!("{icon}   {}", view.label());
        let text = if selected {
            theme::body(label).color(theme::on_accent()).strong()
        } else {
            theme::body(label)
        };
        let nav_height =
            theme::DensityMetrics::for_density(self.session.density).nav_row_height;
        let mut button = egui::Button::new(text)
            .corner_radius(egui::CornerRadius::same(8))
            .min_size(egui::vec2(width, nav_height));
        if selected {
            button = button.fill(theme::accent()).stroke(egui::Stroke::NONE);
        } else {
            button = button
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::NONE);
        }
        let response = ui.add(button).on_hover_text(view.subtitle());
        if response.clicked() {
            self.session.selected_view = view;
        }
    }

    fn render_sidebar_controls(&mut self, ui: &mut egui::Ui) {
        let theme_label = self.session.theme.toggle_label();
        let theme_hint = shortcut_hint(AppShortcut::ToggleTheme).map_or_else(
            || "Switch the workbench colour theme".to_string(),
            |keys| format!("Switch the workbench colour theme ({keys})"),
        );
        let width = ui.available_width();
        if ui
            .add_sized([width, 30.0], egui::Button::new(theme_label))
            .on_hover_text(theme_hint)
            .clicked()
        {
            self.toggle_theme();
        }
        ui.add_space(theme::XS);
        let inspector_label = if self.session.inspector_visible {
            "Hide inspector"
        } else {
            "Show inspector"
        };
        if ui
            .add_sized(
                [width, 30.0],
                egui::Button::selectable(self.session.inspector_visible, inspector_label),
            )
            .on_hover_text("Show or hide the node inspector panel")
            .clicked()
        {
            self.toggle_inspector();
        }
    }
}
