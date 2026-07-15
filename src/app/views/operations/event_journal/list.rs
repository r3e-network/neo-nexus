use eframe::egui;

use crate::app::{
    domain::RuntimeEvent,
    paging::page_count,
    text::truncate_middle,
    theme::{self, DensityMetrics},
    widgets::{fact, list_row_frame, pagination_bar, text_badge},
    NeoNexusApp, EVENT_PAGE_SIZE,
};

use super::super::helpers::event_color;

pub(super) fn render_event_list(app: &mut NeoNexusApp, ui: &mut egui::Ui, events: &[RuntimeEvent]) {
    app.ensure_valid_event_selection(events);
    let total_pages = page_count(events.len(), EVENT_PAGE_SIZE);
    app.operations_ui.event_page = app.operations_ui.event_page.min(total_pages - 1);
    let start = app.operations_ui.event_page * EVENT_PAGE_SIZE;
    let end = (start + EVENT_PAGE_SIZE).min(events.len());
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

    for _ in (end - start)..EVENT_PAGE_SIZE {
        ui.allocate_space(egui::vec2(ui.available_width(), slot));
    }

    render_event_detail(app, ui, events);
}

fn render_event_row(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    event: &RuntimeEvent,
    slot: f32,
) {
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
    egui::Frame::new()
        .fill(theme::card_surface())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
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
