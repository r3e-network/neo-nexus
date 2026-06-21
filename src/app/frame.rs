use std::time::Duration;

use eframe::egui;

use super::{theme::configure_style, NeoNexusApp};

impl eframe::App for NeoNexusApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        configure_style(context, self.theme);
        self.handle_application_shortcuts(context);
        self.drain_alert_delivery_results();
        self.drain_rpc_health_results();
        self.drain_remote_federation_results();
        self.reconcile_supervised_processes();
        self.run_due_watchdog_restarts();
        self.run_due_runtime_upgrade_policy();
        self.refresh_metrics_if_due();
        self.schedule_due_rpc_health_checks();
        self.schedule_due_remote_federation_probes();
        if self.watchdog.has_pending_restart()
            || !self.rpc_health_pending.is_empty()
            || !self.remote_federation_pending.is_empty()
            || self.alert_delivery_pending > 0
        {
            context.request_repaint_after(Duration::from_millis(500));
        } else {
            context.request_repaint_after(Duration::from_secs(1));
        }
        self.ensure_valid_selection();

        egui::TopBottomPanel::top("application_header")
            .resizable(false)
            .exact_height(78.0)
            .show(context, |ui| self.render_application_header(ui));

        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .exact_height(28.0)
            .show(context, |ui| self.render_status_bar(ui));

        egui::SidePanel::left("inventory_panel")
            .resizable(false)
            .exact_width(286.0)
            .show(context, |ui| self.render_inventory_panel(ui));

        egui::SidePanel::right("inspector_panel")
            .resizable(false)
            .exact_width(336.0)
            .show(context, |ui| self.render_inspector_panel(ui));

        egui::CentralPanel::default().show(context, |ui| {
            self.render_workspace(ui);
        });
    }
}
