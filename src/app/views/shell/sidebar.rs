use eframe::egui;

use super::super::super::{
    shortcuts::{labels::shortcut_hint, AppShortcut},
    theme,
    view::View,
    NeoNexusApp,
};

// Pages grouped for a calm, scannable navigation hierarchy.
const NAV_GROUPS: &[(&str, &[View])] = &[
    (
        "Workspace",
        &[View::Summary, View::Operations, View::Monitor, View::Logs],
    ),
    (
        "Nodes",
        &[
            View::Nodes,
            View::Runtimes,
            View::Snapshots,
            View::Plugins,
            View::Config,
        ],
    ),
    (
        "Network",
        &[View::Federation, View::Roles, View::Wallets, View::Alerts],
    ),
    ("System", &[View::Settings]),
];

impl NeoNexusApp {
    pub(in crate::app) fn render_navigation_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(theme::XS);
        ui.label(theme::page_title("NeoNexus"));
        ui.label(theme::muted_body("Neo node operations"));
        ui.add_space(theme::XL);

        // Controls pinned to the bottom; navigation fills the space above.
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
                ui.add_space(theme::LG);
            }
            ui.label(theme::label_caption(*group));
            ui.add_space(theme::XS);
            for &view in *views {
                self.render_nav_item(ui, view);
            }
        }
    }

    fn render_nav_item(&mut self, ui: &mut egui::Ui, view: View) {
        let selected = self.selected_view == view;
        let width = ui.available_width();
        // Phosphor glyph from the shared icon font, set beside the label so the
        // sidebar reads like a macOS source-list: pictogram then title.
        let icon = theme::view_icon_glyph(view);
        let label = format!("{icon}   {}", view.label());
        // The selected page reads as a macOS source-list selection: a solid
        // accent fill with the label drawn in the on-accent foreground, so the
        // active page is unmistakable rather than a subtle egui default tint.
        let text = if selected {
            theme::body(label).color(theme::on_accent()).strong()
        } else {
            theme::body(label)
        };
        let mut button = egui::Button::selectable(selected, text);
        if selected {
            button = button.fill(theme::accent()).stroke(egui::Stroke::NONE);
        }
        let response = ui
            .add_sized([width, 32.0], button)
            .on_hover_text(view.subtitle());
        if response.clicked() {
            self.selected_view = view;
        }
    }

    fn render_sidebar_controls(&mut self, ui: &mut egui::Ui) {
        let theme_label = self.theme.toggle_label();
        let theme_hint = shortcut_hint(AppShortcut::ToggleTheme).map_or_else(
            || "Switch the workbench colour theme".to_string(),
            |keys| format!("Switch the workbench colour theme ({keys})"),
        );
        if ui.button(theme_label).on_hover_text(theme_hint).clicked() {
            self.toggle_theme();
        }
        if ui
            .selectable_label(self.inspector_visible, "Inspector")
            .on_hover_text("Show or hide the node inspector panel")
            .clicked()
        {
            self.toggle_inspector();
        }
    }
}
