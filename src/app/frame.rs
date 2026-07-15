use std::time::Duration;

use eframe::egui;

use super::{theme, theme::configure_style, NeoNexusApp};

impl eframe::App for NeoNexusApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        super::theme::install_icons(context);
        configure_style(context, self.session.theme);
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
        // Mirror operator notices into the toast stack before paint so the same
        // frame that sets a notice also shows it as a coloured chip.
        self.session.toasts.mirror_notice(self.session.notice.as_deref());
        self.session.toasts.expire_due();
        if self.watchdog.has_pending_restart()
            || !self.rpc_health_pending.is_empty()
            || !self.remote_federation_pending.is_empty()
            || self.alert_delivery_pending > 0
            || !self.session.toasts.is_empty()
        {
            context.request_repaint_after(Duration::from_millis(500));
        } else {
            context.request_repaint_after(Duration::from_secs(1));
        }
        self.ensure_valid_selection();
        self.persist_active_view_if_changed();
        self.persist_active_sections_if_changed();
        self.render_application_panels(context);
    }
}

/// A chrome panel frame (sidebar, header, status bar, inspector): a raised
/// surface that lifts off the workspace canvas with the mid-tier panel fill and
/// a hairline border so the chrome reads as distinct from the content area.
fn chrome_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(theme::panel_fill())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(0))
}

impl NeoNexusApp {
    /// Lays out the fixed workbench panels: header, status bar, navigation
    /// sidebar, optional inventory and inspector side panels, and the central
    /// workspace. Extracted from `eframe::App::update` so a headless egui
    /// context can render one real frame for geometry verification without a
    /// live window or the macOS screen-capture permission.
    pub(in crate::app) fn render_application_panels(&mut self, context: &egui::Context) {
        egui::TopBottomPanel::top("application_header")
            .resizable(false)
            .exact_height(60.0)
            .frame(chrome_frame().inner_margin(egui::Margin::symmetric(16, 10)))
            .show(context, |ui| self.render_application_header(ui));

        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .exact_height(28.0)
            .frame(chrome_frame().inner_margin(egui::Margin::symmetric(12, 4)))
            .show(context, |ui| self.render_status_bar(ui));

        egui::SidePanel::left("navigation_panel")
            .resizable(false)
            .exact_width(212.0)
            .frame(chrome_frame().inner_margin(egui::Margin::symmetric(12, 14)))
            .show(context, |ui| self.render_navigation_sidebar(ui));

        if self.session.selected_view.shows_inventory() {
            egui::SidePanel::left("inventory_panel")
                .resizable(true)
                .default_width(248.0)
                .width_range(200.0..=340.0)
                .frame(chrome_frame().inner_margin(egui::Margin::symmetric(14, 14)))
                .show(context, |ui| self.render_inventory_panel(ui));
        }

        if self.session.inspector_visible {
            egui::SidePanel::right("inspector_panel")
                .resizable(true)
                .default_width(320.0)
                .width_range(280.0..=420.0)
                .frame(chrome_frame().inner_margin(egui::Margin::symmetric(16, 14)))
                .show(context, |ui| self.render_inspector_panel(ui));
        }

        let style = context.style();
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&style).inner_margin(egui::Margin::symmetric(22, 18)))
            .show(context, |ui| {
                self.render_workspace(ui);
            });
    }
}

impl NeoNexusApp {
    /// Renders one frame of the workbench against a caller-supplied egui
    /// context. This is the headless entry point used by geometry verification
    /// tests: it installs the icon font and theme style, then lays out the real
    /// panels so their pixel rects can be asserted without a live window or the
    /// macOS screen-capture permission.
    pub fn render_headless_frame(&mut self, context: &egui::Context) {
        super::theme::install_icons(context);
        configure_style(context, self.session.theme);
        self.render_application_panels(context);
    }
}
