use eframe::egui;

use crate::{
    app::domain::{format_bytes, NodeConfig},
    argv::format_argv,
};

use super::{
    super::super::super::{
        format_duration,
        text::{non_empty, short_path, truncate_middle},
        theme::status_color,
        widgets::{fact, render_node_fact_sheet},
        NeoNexusApp,
    },
    truncated_node_name,
};

impl NeoNexusApp {
    pub(super) fn render_selected_node_inspector(&mut self, ui: &mut egui::Ui, node: &NodeConfig) {
        render_node_heading(ui, node);
        ui.separator();

        self.render_node_operational_facts(ui, node);
        self.render_node_process_facts(ui, node);
        self.render_inspector_actions(ui, node);
    }

    fn render_node_operational_facts(&self, ui: &mut egui::Ui, node: &NodeConfig) {
        render_node_fact_sheet(ui, node);
        fact(
            ui,
            "PID",
            &node
                .pid
                .map_or_else(|| "-".to_string(), |pid| pid.to_string()),
        );
        fact(ui, "Binary", &short_path(&node.binary_path, 46));
        fact(
            ui,
            "Args",
            &non_empty(&truncate_middle(&format_argv(&node.args), 46), "-"),
        );
        fact(
            ui,
            "Launch",
            &truncate_middle(&self.launch_plan_for(node).display_command, 46),
        );
        fact(ui, "Workdir", &short_path(&self.node_work_dir(node), 46));
        fact(ui, "Data", &short_path(&self.node_data_dir(node), 46));
        fact(ui, "Log", &short_path(&self.node_log_path(node), 46));
        fact(ui, "Watchdog", &self.watchdog_label(&node.id));
    }

    fn render_node_process_facts(&self, ui: &mut egui::Ui, node: &NodeConfig) {
        if let Some(process) = self.metrics_snapshot.node_process(&node.id) {
            fact(ui, "CPU", &format!("{:.1}%", process.cpu_usage_percent));
            fact(ui, "RSS", &format_bytes(process.memory_bytes));
            fact(
                ui,
                "Uptime",
                &format_duration(std::time::Duration::from_secs(process.run_time_seconds)),
            );
        } else if node.status.is_running() {
            fact(ui, "Process", "not observed");
        }
    }
}

fn render_node_heading(ui: &mut egui::Ui, node: &NodeConfig) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.heading(truncated_node_name(&node.name));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(node.status.label())
                    .color(status_color(node.status))
                    .strong(),
            );
        });
    });
}
