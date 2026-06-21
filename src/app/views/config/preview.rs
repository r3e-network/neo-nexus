use eframe::egui;

use crate::app::{
    domain::RenderedConfig,
    paging::page_count,
    text::truncate_end,
    widgets::{pagination_bar, panel},
    CONFIG_LINES_PER_PAGE,
};

pub(super) fn render_config_preview(
    ui: &mut egui::Ui,
    config_page: &mut usize,
    rendered_config: &anyhow::Result<RenderedConfig>,
) {
    panel(ui, "Paged config preview", |ui| match rendered_config {
        Ok(config) => render_config_lines(ui, config_page, &config.text),
        Err(error) => {
            ui.label(error.to_string());
        }
    });
}

fn render_config_lines(ui: &mut egui::Ui, config_page: &mut usize, text: &str) {
    let lines: Vec<&str> = text.lines().collect();
    let total_pages = page_count(lines.len(), CONFIG_LINES_PER_PAGE);
    *config_page = (*config_page).min(total_pages - 1);
    let start = *config_page * CONFIG_LINES_PER_PAGE;
    let end = (start + CONFIG_LINES_PER_PAGE).min(lines.len());
    let max_chars = ((ui.available_width() / 7.4) as usize).clamp(40, 132);

    pagination_bar(ui, config_page, total_pages, lines.len());
    ui.separator();

    egui::Grid::new("config_lines")
        .striped(true)
        .num_columns(2)
        .min_col_width(36.0)
        .show(ui, |ui| {
            for (line_index, line) in lines.iter().enumerate().take(end).skip(start) {
                ui.monospace(format!("{:>3}", line_index + 1));
                ui.monospace(truncate_end(line, max_chars));
                ui.end_row();
            }

            for _ in (end - start)..CONFIG_LINES_PER_PAGE {
                ui.monospace(" ");
                ui.monospace(" ");
                ui.end_row();
            }
        });
}
