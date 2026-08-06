use eframe::egui;

use egui_phosphor::variants::{regular, Variant};

use crate::app::view::View;

/// The font-definition key `egui_phosphor::add_to_fonts` registers under.
const PHOSPHOR_FONT_KEY: &str = "phosphor";

/// Install the Phosphor icon font into a context's proportional family, once
/// per context. Once installed, the glyphs render inline with ordinary text in
/// the same `RichText`, which is how the sidebar draws an icon next to each
/// label.
///
/// The guard asks the *context* whether it already has the font rather than
/// tracking installation in a process-wide flag. A process-wide guard silently
/// left every context after the first with no icon font at all — every glyph
/// fell back to a tofu box — which is exactly what happens with a second
/// viewport or a second headless render in one process.
/// Must be called from inside a frame: egui has no font state before the first
/// `Context::run`.
pub(in crate::app) fn install(context: &egui::Context) {
    let installed = context.fonts(|fonts| {
        fonts
            .definitions()
            .font_data
            .contains_key(PHOSPHOR_FONT_KEY)
    });
    if installed {
        return;
    }
    let mut fonts = context.fonts(|fonts| fonts.definitions().clone());
    egui_phosphor::add_to_fonts(&mut fonts, Variant::Regular);
    context.set_fonts(fonts);
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

/// Phosphor glyph for an empty state: a muted tray that reads as "nothing here
/// yet" so an empty panel has a focal pictogram above its guidance text, like a
/// macOS empty list, rather than bare words.
pub(in crate::app) fn empty_glyph() -> &'static str {
    regular::TRAY
}

/// Phosphor glyph for the sidebar brand mark: a small connected-nodes pictogram
/// rendered in the accent colour to anchor the workspace identity the way a
/// macOS sidebar marks the owning application.
pub(in crate::app) fn brand_glyph() -> &'static str {
    regular::SHARE_NETWORK
}

#[cfg(test)]
#[path = "../../../tests/unit/app/theme/icons.rs"]
mod tests;
