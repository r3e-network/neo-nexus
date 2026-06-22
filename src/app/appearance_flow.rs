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
    /// the full width when node detail is not needed.
    pub(in crate::app) fn toggle_inspector(&mut self) {
        self.inspector_visible = !self.inspector_visible;
    }
}
