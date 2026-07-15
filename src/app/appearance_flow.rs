use super::*;

impl NeoNexusApp {
    /// Switches between the light and dark workbench themes and persists the
    /// choice so it survives restarts. A persistence failure is surfaced on the
    /// notice line but never blocks the in-session switch.
    pub(in crate::app) fn toggle_theme(&mut self) {
        self.session.theme = self.session.theme.toggled();
        if let Err(error) = self.repository.save_app_dark_mode(self.session.theme.is_dark()) {
            self.session.notice = Some(format!("Theme preference not saved: {error}"));
        }
    }

    /// Persists the operator density preference and applies control metrics
    /// on the next frame via `configure_style_with_density`.
    pub(in crate::app) fn set_ui_density(&mut self, density: UiDensity) {
        self.session.density = density;
        if let Err(error) = self
            .repository
            .save_app_ui_density(density.persist_key())
        {
            self.session.notice = Some(format!("Density preference not saved: {error}"));
        }
    }

    /// Shows or hides the right-hand inspector panel so the workspace can use
    /// the full width when node detail is not needed. The choice is persisted so
    /// the layout is restored on the next launch.
    pub(in crate::app) fn toggle_inspector(&mut self) {
        self.session.inspector_visible = !self.session.inspector_visible;
        if let Err(error) = self
            .repository
            .save_app_inspector_visible(self.session.inspector_visible)
        {
            self.session.notice = Some(format!("Inspector preference not saved: {error}"));
        }
    }

    /// Persists the active workspace view when it changes so the app reopens on
    /// the page the operator left. Called once per frame; the SQLite write only
    /// happens on an actual change.
    pub(in crate::app) fn persist_active_view_if_changed(&mut self) {
        if self.session.selected_view == self.session.persisted_view {
            return;
        }
        self.session.persisted_view = self.session.selected_view;
        if let Err(error) = self
            .repository
            .save_workspace_last_view(self.session.selected_view.persist_key())
        {
            self.session.notice = Some(format!("Workspace view not saved: {error}"));
        }
    }

    /// Switches the active workspace view programmatically. Mirrors selecting a
    /// page in the sidebar, so automation, scripting, and verification harnesses
    /// can drive the workbench to a specific page without simulating a click.
    pub fn select_view(&mut self, view: View) {
        self.session.selected_view = view;
    }
}
