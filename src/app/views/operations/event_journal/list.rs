use eframe::egui;

use crate::app::widgets::inset_card;

use crate::app::{
    domain::RuntimeEvent,
    paging::{page_count, rows_that_fit},
    text::truncate_middle,
    theme::{self, DensityMetrics},
    widgets::{fact, list_row_frame, pagination_bar, text_badge},
    NeoNexusApp, EVENT_PAGE_SIZE,
};

use super::super::helpers::event_color;

/// Pitch of one populated journal row plus the `XS` gap after it. `journal_slot`
/// is the *empty*-slot reserve; a filled row carries a badge beside two text
/// lines and outgrows it, measuring 96pt at the 1280x820 design window in the
/// narrow column the open inspector leaves (81pt with the inspector hidden).
/// Paging on the slot alone would still push rows off the panel.
const EVENT_ROW_PITCH: f32 = 96.0;

/// Chrome this function draws around the rows: the pagination bar with its
/// trailing `SM` gap above (47pt) and the event detail card below (264pt). The
/// card is unconditional — `ensure_valid_event_selection` always leaves an event
/// selected for a non-empty page.
const EVENT_CHROME: f32 = 47.0 + 264.0;

pub(super) fn render_event_list(app: &mut NeoNexusApp, ui: &mut egui::Ui, events: &[RuntimeEvent]) {
    app.ensure_valid_event_selection(events);
    // Read before the pagination bar goes in, so the bar belongs to the reserve.
    let page_size =
        rows_that_fit(ui.available_height(), EVENT_ROW_PITCH, EVENT_CHROME).min(EVENT_PAGE_SIZE);
    let total_pages = page_count(events.len(), page_size);
    app.operations_ui.event_page = app.operations_ui.event_page.min(total_pages - 1);
    let start = app.operations_ui.event_page * page_size;
    let end = (start + page_size).min(events.len());
    let slot = DensityMetrics::COMFORTABLE.journal_slot;

    pagination_bar(
        ui,
        &mut app.operations_ui.event_page,
        total_pages,
        events.len(),
    );
    ui.add_space(theme::SM);

    for event in events.iter().take(end).skip(start) {
        render_event_row(app, ui, event, slot);
        ui.add_space(theme::XS);
    }

    for _ in (end - start)..page_size {
        ui.allocate_space(egui::vec2(ui.available_width(), slot));
    }

    render_event_detail(app, ui, events);
}

fn render_event_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, event: &RuntimeEvent, slot: f32) {
    let selected = app.operations_ui.selected_event == Some(event.id);
    if list_row_frame(ui, selected, Some(slot), |ui| {
        ui.horizontal(|ui| {
            text_badge(ui, event.severity.label(), event_color(event.severity));
            ui.add_space(theme::SM);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        theme::body(truncate_middle(
                            event.node_name.as_deref().unwrap_or("workspace"),
                            22,
                        ))
                        .strong(),
                    );
                    ui.label(theme::muted_body(event.kind.label()));
                });
                ui.label(theme::muted_body(truncate_middle(&event.message, 64)));
            });
        });
    }) {
        app.select_event(event);
    }
}

fn render_event_detail(app: &NeoNexusApp, ui: &mut egui::Ui, events: &[RuntimeEvent]) {
    let Some(event) = app.selected_event_from(events) else {
        return;
    };

    ui.add_space(theme::SM);
    inset_card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(theme::label_caption("Event detail"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(theme::muted_body(format!("#{}", event.id)));
            });
        });
        ui.add_space(theme::SM);
        fact(ui, "Kind", event.kind.label());
        fact(
            ui,
            "Node",
            event.node_name.as_deref().unwrap_or("workspace"),
        );
        fact(ui, "Time", &event.occurred_at_unix.to_string());
        ui.horizontal(|ui| {
            ui.label(theme::muted_body("Severity"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                text_badge(ui, event.severity.label(), event_color(event.severity));
            });
        });
        ui.add_space(theme::SM);
        ui.label(theme::body(event.message.as_str()));
    });
}
