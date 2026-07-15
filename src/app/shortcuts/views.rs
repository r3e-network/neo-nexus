use eframe::egui;

use super::View;

pub(super) fn next_view(view: View) -> View {
    shifted_primary(view, 1)
}

pub(super) fn previous_view(view: View) -> View {
    shifted_primary(view, View::PRIMARY.len() - 1)
}

fn shifted_primary(view: View, offset: usize) -> View {
    let primary = view.primary_nav();
    let index = View::PRIMARY
        .iter()
        .position(|candidate| *candidate == primary)
        .unwrap_or(0);
    View::PRIMARY[(index + offset) % View::PRIMARY.len()]
}

pub(super) fn numbered_view_shortcut(key: egui::Key) -> Option<View> {
    NUMBERED_VIEW_KEYS
        .iter()
        .position(|candidate| *candidate == key)
        .and_then(|index| View::PRIMARY.get(index))
        .copied()
}

const NUMBERED_VIEW_KEYS: [egui::Key; 6] = [
    egui::Key::Num1,
    egui::Key::Num2,
    egui::Key::Num3,
    egui::Key::Num4,
    egui::Key::Num5,
    egui::Key::Num6,
];
