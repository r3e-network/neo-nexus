use std::time::Duration;

use eframe::egui;

use super::{theme::configure_style, NeoNexusApp};

impl eframe::App for NeoNexusApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        super::theme::install_icons(context);
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
        self.persist_active_view_if_changed();
        self.persist_active_sections_if_changed();

        egui::TopBottomPanel::top("application_header")
            .resizable(false)
            .exact_height(60.0)
            .show(context, |ui| self.render_application_header(ui));

        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .exact_height(28.0)
            .show(context, |ui| self.render_status_bar(ui));

        let style = context.style();

        egui::SidePanel::left("navigation_panel")
            .resizable(false)
            .exact_width(212.0)
            .frame(
                egui::Frame::side_top_panel(&style).inner_margin(egui::Margin::symmetric(12, 12)),
            )
            .show(context, |ui| self.render_navigation_sidebar(ui));

        if self.selected_view.shows_inventory() {
            egui::SidePanel::left("inventory_panel")
                .resizable(true)
                .default_width(248.0)
                .width_range(200.0..=340.0)
                .frame(
                    egui::Frame::side_top_panel(&style)
                        .inner_margin(egui::Margin::symmetric(14, 14)),
                )
                .show(context, |ui| self.render_inventory_panel(ui));
        }

        if self.inspector_visible {
            egui::SidePanel::right("inspector_panel")
                .resizable(true)
                .default_width(320.0)
                .width_range(280.0..=420.0)
                .frame(
                    egui::Frame::side_top_panel(&style)
                        .inner_margin(egui::Margin::symmetric(16, 14)),
                )
                .show(context, |ui| self.render_inspector_panel(ui));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&style).inner_margin(egui::Margin::symmetric(22, 18)))
            .show(context, |ui| {
                self.render_workspace(ui);
            });
    }
}
