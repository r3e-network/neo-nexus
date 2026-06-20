use eframe::egui;

use super::{view::View, NeoNexusApp, NODE_PAGE_SIZE};

mod commands;
mod input;
pub(in crate::app) mod labels;
mod nodes;
mod views;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum AppShortcut {
    ReloadWorkspace,
    NewNode,
    StartSelectedNode,
    StopSelectedNode,
    RestartSelectedNode,
    ToggleSelectedNode,
    PreviousNode,
    NextNode,
    PreviousNodePage,
    NextNodePage,
    FirstNode,
    LastNode,
    NextView,
    PreviousView,
    SelectView(View),
}

impl NeoNexusApp {
    pub(super) fn handle_application_shortcuts(&mut self, context: &egui::Context) {
        if context.wants_keyboard_input() {
            return;
        }

        if let Some(shortcut) = input::consume_application_shortcut(context) {
            self.apply_application_shortcut(shortcut);
        }
    }
}
