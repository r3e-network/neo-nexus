use super::*;

const PLAN_ROWS: usize = 7;

impl NeoNexusApp {
    pub(in crate::app::views::roles) fn render_selected_role_plan(&self, ui: &mut egui::Ui) {
        let Some(node) = self.selected_node().cloned() else {
            empty_state(ui, "No node selected", "Choose a node from Inventory.");
            return;
        };

        let plan = RolePlanner::plan(&node, self.selected_role);
        let states = self
            .repository
            .list_plugin_states(&node.id)
            .unwrap_or_default();

        fact(ui, "Role", plan.role.label());
        fact(ui, "Runtime", &plan.node_type.to_string());
        fact(ui, "Storage", &plan.storage_engine.to_string());
        ui.separator();

        if plan.plugin_changes.is_empty() {
            for note in plan.notes.iter().take(PLAN_ROWS) {
                ui.label(*note);
            }
            return;
        }

        egui::Grid::new("role_plan_changes")
            .striped(true)
            .min_col_width(72.0)
            .show(ui, |ui| {
                ui.strong("Plugin");
                ui.strong("Current");
                ui.strong("Target");
                ui.strong("Reason");
                ui.end_row();

                for row in 0..PLAN_ROWS {
                    if let Some(change) = plan.plugin_changes.get(row) {
                        let current = plugin_enabled(&states, change.plugin_id);
                        ui.label(change.plugin_id.to_string());
                        ui.label(if current { "On" } else { "Off" });
                        ui.label(if change.enabled { "On" } else { "Off" });
                        ui.label(truncate_middle(change.reason, 48));
                    } else {
                        for _ in 0..4 {
                            ui.label(" ");
                        }
                    }
                    ui.end_row();
                }
            });

        ui.separator();
        for note in plan.notes.iter().take(2) {
            ui.label(egui::RichText::new(*note).color(muted_text()));
        }
    }
}
