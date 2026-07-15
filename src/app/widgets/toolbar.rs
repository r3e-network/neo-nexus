use eframe::egui;

use crate::app::theme;

use super::controls::{primary_button, secondary_button, secondary_button_enabled};

/// One toolbar action. Primary actions render as accent-filled buttons.
#[derive(Debug, Clone)]
pub(in crate::app) struct ToolbarAction {
    pub(in crate::app) id: &'static str,
    pub(in crate::app) label: &'static str,
    pub(in crate::app) enabled: bool,
    pub(in crate::app) primary: bool,
    pub(in crate::app) hint: Option<&'static str>,
}

impl ToolbarAction {
    pub(in crate::app) fn primary(id: &'static str, label: &'static str) -> Self {
        Self {
            id,
            label,
            enabled: true,
            primary: true,
            hint: None,
        }
    }

    pub(in crate::app) fn secondary(id: &'static str, label: &'static str) -> Self {
        Self {
            id,
            label,
            enabled: true,
            primary: false,
            hint: None,
        }
    }

    pub(in crate::app) fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub(in crate::app) fn hint(mut self, hint: &'static str) -> Self {
        self.hint = Some(hint);
        self
    }
}

/// Renders a wrapping toolbar of primary then secondary actions.
/// Returns the `id` of the action clicked this frame, if any.
pub(in crate::app) fn toolbar(ui: &mut egui::Ui, actions: &[ToolbarAction]) -> Option<&'static str> {
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = theme::SM;
        ui.spacing_mut().item_spacing.y = theme::XS;
        for action in actions {
            let response = if action.primary {
                ui.add_enabled_ui(action.enabled, |ui| primary_button(ui, action.label))
                    .inner
            } else if action.enabled {
                secondary_button(ui, action.label)
            } else {
                secondary_button_enabled(ui, action.label, false)
            };
            let response = match action.hint {
                Some(hint) => response.on_hover_text(hint),
                None => response,
            };
            if response.clicked() {
                clicked = Some(action.id);
            }
        }
    });
    clicked
}
