//! Legacy title+segment header. Prefer [`super::page_chrome::page_chrome`] with
//! `title: None` on primary surfaces so the shell header is not duplicated.

#![allow(dead_code)]

use eframe::egui;

use super::page_chrome::page_chrome;

/// Thin wrapper retained for any residual callers. Primaries should not pass a title.
pub(in crate::app) fn page_header(
    ui: &mut egui::Ui,
    title: &str,
    _subtitle: Option<&str>,
    segments: Option<(&[&str], &mut usize)>,
) -> bool {
    // Subtitle ignored: shell owns copy. Title only if non-empty for rare cases.
    let title_opt = if title.is_empty() { None } else { Some(title) };
    page_chrome(ui, title_opt, segments)
}
