use eframe::egui;

use crate::logs::{LogReader, LogSnapshot};

use super::{
    super::super::{
        paging::page_count,
        text::truncate_end,
        theme::muted_text,
        widgets::{empty_state, pagination_bar},
        NeoNexusApp, LOG_LINES_PER_PAGE,
    },
    diagnosis::render_log_diagnosis,
};

pub(super) fn render_log_output(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    snapshot: &anyhow::Result<LogSnapshot>,
) {
    match snapshot {
        Ok(snapshot) => render_log_lines(app, ui, snapshot),
        Err(error) => {
            ui.label(error.to_string());
        }
    }
}

fn render_log_lines(app: &mut NeoNexusApp, ui: &mut egui::Ui, snapshot: &LogSnapshot) {
    let diagnosis = LogReader::diagnose(snapshot);
    render_log_diagnosis(ui, &diagnosis);

    if !snapshot.exists {
        empty_state(
            ui,
            "No log file",
            "Start the selected node to create runtime output.",
        );
        return;
    }

    let lines = LogReader::filtered_lines(snapshot, &app.log_query);

    if snapshot.lines.is_empty() {
        empty_state(
            ui,
            "Empty log",
            "No output has been captured for this node yet.",
        );
        return;
    }

    if lines.is_empty() {
        empty_state(ui, "No matches", "Adjust the search query.");
        return;
    }

    let total_pages = page_count(lines.len(), LOG_LINES_PER_PAGE);
    if app.log_follow_tail {
        app.log_page = total_pages - 1;
    }
    app.log_page = app.log_page.min(total_pages - 1);
    let start = app.log_page * LOG_LINES_PER_PAGE;
    let end = (start + LOG_LINES_PER_PAGE).min(lines.len());
    let max_chars = ((ui.available_width() / 7.4) as usize).clamp(48, 180);

    pagination_bar(ui, &mut app.log_page, total_pages, lines.len());
    if app.log_page + 1 < total_pages {
        app.log_follow_tail = false;
    }
    if snapshot.truncated {
        ui.label(
            egui::RichText::new("Showing the retained tail window of a larger log file.")
                .color(muted_text()),
        );
    }
    ui.separator();

    egui::Grid::new("log_lines")
        .striped(true)
        .num_columns(2)
        .min_col_width(42.0)
        .show(ui, |ui| {
            for line in lines.iter().take(end).skip(start) {
                ui.monospace(format!("{:>4}", line.number));
                ui.monospace(truncate_end(&line.text, max_chars));
                ui.end_row();
            }

            for _ in (end - start)..LOG_LINES_PER_PAGE {
                ui.monospace(" ");
                ui.monospace(" ");
                ui.end_row();
            }
        });
}
