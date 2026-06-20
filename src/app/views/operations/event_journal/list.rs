use eframe::egui;

use crate::{
    app::{
        paging::page_count,
        text::truncate_middle,
        theme::muted_text,
        widgets::{fact, pagination_bar},
        NeoNexusApp, EVENT_PAGE_SIZE,
    },
    events::RuntimeEvent,
};

use super::super::helpers::event_color;

pub(super) fn render_event_list(app: &mut NeoNexusApp, ui: &mut egui::Ui, events: &[RuntimeEvent]) {
    app.ensure_valid_event_selection(events);
    let total_pages = page_count(events.len(), EVENT_PAGE_SIZE);
    app.event_page = app.event_page.min(total_pages - 1);
    let start = app.event_page * EVENT_PAGE_SIZE;
    let end = (start + EVENT_PAGE_SIZE).min(events.len());

    pagination_bar(ui, &mut app.event_page, total_pages, events.len());
    ui.separator();

    for event in events.iter().take(end).skip(start) {
        render_event_row(app, ui, event);
    }

    for _ in (end - start)..EVENT_PAGE_SIZE {
        ui.allocate_space(egui::vec2(ui.available_width(), 38.0));
    }

    render_event_detail(app, ui, events);
}

fn render_event_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, event: &RuntimeEvent) {
    let label = format!(
        "{}  {}  {}\n{}",
        event.severity.label(),
        truncate_middle(event.node_name.as_deref().unwrap_or("workspace"), 18),
        event.occurred_at_unix,
        truncate_middle(&format!("{} {}", event.kind.label(), event.message), 48),
    );
    let selected = app.selected_event == Some(event.id);
    if ui
        .add_sized(
            [ui.available_width(), 38.0],
            egui::Button::new(label).selected(selected),
        )
        .on_hover_text(&event.message)
        .clicked()
    {
        app.select_event(event);
    }
}

fn render_event_detail(app: &NeoNexusApp, ui: &mut egui::Ui, events: &[RuntimeEvent]) {
    let Some(event) = app.selected_event_from(events) else {
        return;
    };

    ui.separator();
    ui.horizontal(|ui| {
        ui.strong("Event detail");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(format!("#{}", event.id)).color(muted_text()));
        });
    });
    fact(ui, "Kind", event.kind.label());
    fact(
        ui,
        "Node",
        event.node_name.as_deref().unwrap_or("workspace"),
    );
    fact(ui, "Time", &event.occurred_at_unix.to_string());
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Severity").color(muted_text()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(event.severity.label())
                    .strong()
                    .color(event_color(event.severity)),
            );
        });
    });
    ui.label(truncate_middle(&event.message, 72))
        .on_hover_text(event.message);
}
