use super::*;

impl NeoNexusApp {
    /// Switches between the light and dark workbench themes and persists the
    /// choice so it survives restarts. A persistence failure is surfaced on the
    /// notice line but never blocks the in-session switch.
    pub(in crate::app) fn toggle_theme(&mut self) {
        self.theme = self.theme.toggled();
        if let Err(error) = self.repository.save_app_dark_mode(self.theme.is_dark()) {
            self.notice = Some(format!("Theme preference not saved: {error}"));
        }
    }

    /// Shows or hides the right-hand inspector panel so the workspace can use
    /// the full width when node detail is not needed. The choice is persisted so
    /// the layout is restored on the next launch.
    pub(in crate::app) fn toggle_inspector(&mut self) {
        self.inspector_visible = !self.inspector_visible;
        if let Err(error) = self
            .repository
            .save_app_inspector_visible(self.inspector_visible)
        {
            self.notice = Some(format!("Inspector preference not saved: {error}"));
        }
    }

    /// Persists the active workspace view when it changes so the app reopens on
    /// the page the operator left. Called once per frame; the SQLite write only
    /// happens on an actual change.
    pub(in crate::app) fn persist_active_view_if_changed(&mut self) {
        if self.selected_view == self.persisted_view {
            return;
        }
        self.persisted_view = self.selected_view;
        if let Err(error) = self
            .repository
            .save_workspace_last_view(self.selected_view.persist_key())
        {
            self.notice = Some(format!("Workspace view not saved: {error}"));
        }
    }

    /// Switches the active workspace view programmatically. Mirrors selecting a
    /// page in the sidebar, so automation, scripting, and verification harnesses
    /// can drive the workbench to a specific page without simulating a click.
    pub fn select_view(&mut self, view: View) {
        self.selected_view = view;
    }
}
