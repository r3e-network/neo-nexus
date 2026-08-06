use eframe::egui;

use crate::{
    app::domain::{format_bytes, NodeConfig},
    argv::format_argv,
};

use super::super::super::super::{
    format_duration,
    text::{non_empty, short_path, truncate_middle},
    theme,
    widgets::{card, fact, metric_grid},
    NeoNexusApp,
};

/// How wide a path value may render before it is middle-truncated. Tuned to the
/// inspector's 280–420pt panel so a value never pushes its label off the row.
const PATH_CHARS: usize = 34;

impl NeoNexusApp {
    /// Definition facts as a two-column grid, then the lifecycle actions.
    pub(super) fn render_inspector_overview(&mut self, ui: &mut egui::Ui, node: &NodeConfig) {
        card(ui, "Definition", None, |ui| {
            metric_grid(
                ui,
                &[
                    ("Type", node.node_type.to_string()),
                    ("Network", node.network.to_string()),
                    ("Version", node.runtime_version.clone()),
                    ("Storage", node.storage_engine.to_string()),
                    ("RPC port", node.rpc_port.to_string()),
                    ("P2P port", node.p2p_port.to_string()),
                    ("WebSocket", optional_port(node.ws_port)),
                ],
            );
        });
        ui.add_space(theme::SM);
        card(ui, "Actions", None, |ui| {
            self.render_inspector_actions(ui, node);
        });
    }

    /// Filesystem locations. Kept in their own section because a path value is
    /// far too wide to share a row with a second column.
    pub(super) fn render_inspector_paths(&mut self, ui: &mut egui::Ui, node: &NodeConfig) {
        card(ui, "Paths", None, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            fact(ui, "Binary", &short_path(&node.binary_path, PATH_CHARS));
            fact(
                ui,
                "Args",
                &non_empty(&truncate_middle(&format_argv(&node.args), PATH_CHARS), "—"),
            );
            fact(
                ui,
                "Launch",
                &truncate_middle(&self.launch_plan_for(node).display_command, PATH_CHARS),
            );
            fact(
                ui,
                "Workdir",
                &short_path(&self.node_work_dir(node), PATH_CHARS),
            );
            fact(
                ui,
                "Data",
                &short_path(&self.node_data_dir(node), PATH_CHARS),
            );
            fact(
                ui,
                "Log",
                &short_path(&self.node_log_path(node), PATH_CHARS),
            );
            fact(ui, "Watchdog", &self.watchdog_label(&node.id));
        });
    }

    /// Live process telemetry for this node, then the application runtime card.
    pub(super) fn render_inspector_process(&mut self, ui: &mut egui::Ui, node: &NodeConfig) {
        card(ui, "Process", None, |ui| {
            if let Some(process) = self.metrics_snapshot.node_process(&node.id) {
                metric_grid(
                    ui,
                    &[
                        ("CPU", format!("{:.1}%", process.cpu_usage_percent)),
                        ("Memory", format_bytes(process.memory_bytes)),
                        (
                            "Uptime",
                            format_duration(std::time::Duration::from_secs(
                                process.run_time_seconds,
                            )),
                        ),
                        ("Status", node.status.label().to_string()),
                        ("PID", optional_pid(node.pid)),
                    ],
                );
            } else {
                let observed = if node.status.is_running() {
                    "running, not observed"
                } else {
                    "stopped"
                };
                metric_grid(ui, &[("Process", observed.to_string())]);
            }
        });
        ui.add_space(theme::SM);
        self.render_runtime_facts(ui);
    }
}

fn optional_port(port: Option<u16>) -> String {
    port.map_or_else(|| "—".to_string(), |port| port.to_string())
}

fn optional_pid(pid: Option<u32>) -> String {
    pid.map_or_else(|| "—".to_string(), |pid| pid.to_string())
}
