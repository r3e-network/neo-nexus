use super::*;

pub(in crate::app::views::roles::private_network) fn render_plan_actions(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    can_create_nodes: bool,
    launch_pack_ready: bool,
    signer_handoff_valid: bool,
) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(can_create_nodes, egui::Button::new("Create Nodes"))
            .clicked()
        {
            app.materialize_private_network_plan();
        }
        if ui
            .add_enabled(
                launch_pack_ready && signer_handoff_valid,
                egui::Button::new("Export Launch Pack"),
            )
            .clicked()
        {
            app.export_private_network_launch_pack();
        }
        if ui
            .add_enabled(
                app.private_network_last_export_root.is_some(),
                egui::Button::new("Revalidate Pack"),
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
        if ui.button("Save Policy").clicked() {
            app.save_private_network_sidecar_execution_policy();
        }
        if ui
            .add_enabled(
                app.private_network_last_export_root.is_some(),
                egui::Button::new("Load Sidecars"),
            )
            .clicked()
        {
            app.refresh_private_network_launch_pack_sidecars();
        }
        if ui
            .add_enabled(
                app.private_network_last_export_root.is_some(),
                egui::Button::new("Start Sidecars"),
            )
            .clicked()
        {
            app.start_private_network_launch_pack_sidecars();
        }
        if ui
            .add_enabled(
                !app.private_network_sidecar_pids.is_empty(),
                egui::Button::new("Stop Sidecars"),
            )
            .clicked()
        {
            app.stop_private_network_launch_pack_sidecars();
        }
        if ui
            .add_enabled(
                app.private_network_sidecar_report.is_some()
                    || app.private_network_last_export_root.is_some(),
                egui::Button::new("Check Health"),
            )
            .clicked()
        {
            app.check_private_network_launch_pack_sidecar_health();
        }
    });
}
