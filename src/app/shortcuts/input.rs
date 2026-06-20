use eframe::egui;

use super::{views::numbered_view_shortcut, AppShortcut};

pub(super) fn consume_application_shortcut(context: &egui::Context) -> Option<AppShortcut> {
    context.input_mut(|input| {
        if input.consume_key(egui::Modifiers::ALT, egui::Key::Home) {
            return Some(AppShortcut::FirstNode);
        }
        if input.consume_key(egui::Modifiers::ALT, egui::Key::End) {
            return Some(AppShortcut::LastNode);
        }
        if input.consume_key(egui::Modifiers::ALT, egui::Key::PageUp) {
            return Some(AppShortcut::PreviousNodePage);
        }
        if input.consume_key(egui::Modifiers::ALT, egui::Key::PageDown) {
            return Some(AppShortcut::NextNodePage);
        }
        if input.consume_key(egui::Modifiers::ALT, egui::Key::ArrowUp) {
            return Some(AppShortcut::PreviousNode);
        }
        if input.consume_key(egui::Modifiers::ALT, egui::Key::ArrowDown) {
            return Some(AppShortcut::NextNode);
        }
        if input.consume_key(egui::Modifiers::COMMAND, egui::Key::OpenBracket) {
            return Some(AppShortcut::PreviousView);
        }
        if input.consume_key(egui::Modifiers::COMMAND, egui::Key::CloseBracket) {
            return Some(AppShortcut::NextView);
        }
        if input.consume_key(egui::Modifiers::COMMAND, egui::Key::R)
            || input.consume_key(egui::Modifiers::NONE, egui::Key::F5)
        {
            return Some(AppShortcut::ReloadWorkspace);
        }
        if input.consume_key(egui::Modifiers::COMMAND, egui::Key::N) {
            return Some(AppShortcut::NewNode);
        }
        if input.consume_key(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::Enter,
        ) {
            return Some(AppShortcut::RestartSelectedNode);
        }
        if input.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter) {
            return Some(AppShortcut::ToggleSelectedNode);
        }

        for key in NUMBERED_VIEW_KEYS {
            if input.consume_key(egui::Modifiers::COMMAND, key) {
                return numbered_view_shortcut(key).map(AppShortcut::SelectView);
            }
        }

        None
    })
}

const NUMBERED_VIEW_KEYS: [egui::Key; 9] = [
    egui::Key::Num1,
    egui::Key::Num2,
    egui::Key::Num3,
    egui::Key::Num4,
    egui::Key::Num5,
    egui::Key::Num6,
    egui::Key::Num7,
    egui::Key::Num8,
    egui::Key::Num9,
];
