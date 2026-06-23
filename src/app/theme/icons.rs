use std::sync::OnceLock;

use eframe::egui;

use egui_phosphor::variants::{regular, Variant};

use crate::app::view::View;

static ICONS_INSTALLED: OnceLock<()> = OnceLock::new();

/// Install the Phosphor icon font into the egui proportional family once per
/// process. The guard makes repeated per-frame calls a cheap read, and
/// `set_fonts` itself short-circuits when the definitions are unchanged. Once
/// installed, the glyphs render inline with ordinary text in the same
/// `RichText`, which is how the sidebar draws an icon next to each label.
pub(in crate::app) fn install(context: &egui::Context) {
    if ICONS_INSTALLED.get().is_some() {
        return;
    }
    let mut fonts = context.fonts(|fonts| fonts.definitions().clone());
    egui_phosphor::add_to_fonts(&mut fonts, Variant::Regular);
    context.set_fonts(fonts);
    let _ = ICONS_INSTALLED.set(());
}

/// Phosphor glyph for a workspace view, chosen to read as a macOS sidebar
/// pictogram. Returns an empty string only if the view is unmapped, which the
/// sidebar renders as plain text.
pub(in crate::app) fn glyph(view: View) -> &'static str {
    match view {
        View::Summary => regular::HOUSE,
        View::Operations => regular::LIST_CHECKS,
        View::Monitor => regular::GAUGE,
        View::Logs => regular::TERMINAL,
        View::Nodes => regular::CUBE,
        View::Runtimes => regular::STACK,
        View::Snapshots => regular::DATABASE,
        View::Plugins => regular::PLUGS_CONNECTED,
        View::Config => regular::GEAR_FINE,
        View::Federation => regular::GLOBE,
        View::Roles => regular::TREE_STRUCTURE,
        View::Wallets => regular::WALLET,
        View::Alerts => regular::BELL,
        View::Settings => regular::GEAR_SIX,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app/theme/icons.rs"]
mod tests;
