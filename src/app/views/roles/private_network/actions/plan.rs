use super::*;

use crate::app::widgets;

pub(in crate::app::views::roles::private_network) fn render_plan_actions(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    can_create_nodes: bool,
    launch_pack_ready: bool,
    signer_handoff_valid: bool,
) {
    ui.horizontal(|ui| {
        if widgets::secondary_button_enabled(ui, "Create Nodes", can_create_nodes).clicked() {
            app.materialize_private_network_plan();
        }
        if widgets::secondary_button_enabled(
            ui,
            "Export Launch Pack",
            launch_pack_ready && signer_handoff_valid,
        )
        .clicked()
        {
            app.export_private_network_launch_pack();
        }
        if widgets::secondary_button_enabled(
            ui,
            "Revalidate Pack",
            app.private_network_last_export_root.is_some(),
        )
        .clicked()
        {
            app.revalidate_private_network_launch_pack();
        }
    });
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut app.private_network_allow_external_sidecars,
            "Allow External",
        );
        if widgets::secondary_button(ui, "Save Policy").clicked() {
            app.save_private_network_sidecar_execution_policy();
        }
        if widgets::secondary_button_enabled(
            ui,
            "Load Sidecars",
            app.private_network_last_export_root.is_some(),
        )
        .clicked()
        {
            app.refresh_private_network_launch_pack_sidecars();
        }
        if widgets::secondary_button_enabled(
            ui,
            "Start Sidecars",
            app.private_network_last_export_root.is_some(),
        )
        .clicked()
        {
            app.start_private_network_launch_pack_sidecars();
        }
        if widgets::secondary_button_enabled(
            ui,
            "Stop Sidecars",
            !app.private_network_sidecar_pids.is_empty(),
        )
        .clicked()
        {
            app.stop_private_network_launch_pack_sidecars();
        }
        if widgets::secondary_button_enabled(
            ui,
            "Check Health",
            app.private_network_sidecar_report.is_some()
                || app.private_network_last_export_root.is_some(),
        )
        .clicked()
        {
            app.check_private_network_launch_pack_sidecar_health();
        }
    });
}
