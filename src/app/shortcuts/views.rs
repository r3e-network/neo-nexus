use eframe::egui;

use super::View;

pub(super) fn next_view(view: View) -> View {
    shifted_view(view, 1)
}

pub(super) fn previous_view(view: View) -> View {
    shifted_view(view, View::ALL.len() - 1)
}

fn shifted_view(view: View, offset: usize) -> View {
    let index = View::ALL
        .iter()
        .position(|candidate| *candidate == view)
        .unwrap_or(0);
    View::ALL[(index + offset) % View::ALL.len()]
}

pub(super) fn numbered_view_shortcut(key: egui::Key) -> Option<View> {
    NUMBERED_VIEW_KEYS
        .iter()
        .position(|candidate| *candidate == key)
        .and_then(|index| View::ALL.get(index))
        .copied()
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
