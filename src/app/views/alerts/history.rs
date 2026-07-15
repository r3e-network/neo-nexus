use eframe::egui;

use crate::app::domain::{
    filter_alert_deliveries, AlertDelivery, AlertDeliveryFilter, AlertDeliveryStatus, NodeStatus,
};

use super::super::super::{
    paging::page_count,
    text::truncate_middle,
    theme::{muted_text, status_color},
    widgets::{chip_pill, empty_state, grid_header, pagination_bar},
    NeoNexusApp, ALERT_DELIVERY_PAGE_SIZE,
};

pub(super) fn render_alert_delivery_history(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    deliveries: &[AlertDelivery],
) {
    render_filter_bar(app, ui);
    let filter = AlertDeliveryFilter::new(
        app.async_bus.alert_delivery_status_filter,
        app.async_bus.alert_delivery_query.as_str(),
    );
    let filter_active = !filter.is_empty();
    let deliveries = filter_alert_deliveries(deliveries, &filter);

    if deliveries.is_empty() {
        empty_state(ui, empty_title(filter_active), empty_message(filter_active));
        return;
    }

    let total_pages = page_count(deliveries.len(), ALERT_DELIVERY_PAGE_SIZE);
    app.async_bus.alert_delivery_page = app.async_bus.alert_delivery_page.min(total_pages - 1);
    let start = app.async_bus.alert_delivery_page * ALERT_DELIVERY_PAGE_SIZE;
    let end = (start + ALERT_DELIVERY_PAGE_SIZE).min(deliveries.len());

    pagination_bar(
        ui,
        &mut app.async_bus.alert_delivery_page,
        total_pages,
        deliveries.len(),
    );
    ui.separator();

    egui::Grid::new("alert_delivery_history")
        .num_columns(6)
        .spacing([10.0, 6.0])
        .show(ui, |ui| {
            grid_header(
                ui,
                &["Time", "Status", "HTTP", "Provider", "Target", "Message"],
            );

            for delivery in deliveries.iter().take(end).skip(start) {
                ui.label(delivery.attempted_at_unix.to_string());
                ui.label(
                    egui::RichText::new(delivery.status.label())
                        .strong()
                        .color(alert_delivery_color(delivery.status)),
                );
                ui.label(
                    delivery
                        .http_status
                        .map_or_else(|| "-".to_string(), |status| status.to_string()),
                );
                ui.label(delivery.route_label.as_str());
                ui.label(truncate_middle(&delivery.target, 18));
                ui.label(truncate_middle(&delivery.message, 42));
                ui.end_row();
            }
        });

    for _ in (end - start)..ALERT_DELIVERY_PAGE_SIZE {
        ui.allocate_space(egui::vec2(ui.available_width(), 24.0));
    }
}

fn render_filter_bar(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Status").color(muted_text()));
        chip_pill(ui, |ui| {
            filter_button(app, ui, "All", None);
            filter_button(app, ui, "Delivered", Some(AlertDeliveryStatus::Delivered));
            filter_button(app, ui, "Failed", Some(AlertDeliveryStatus::Failed));
            filter_button(app, ui, "Skipped", Some(AlertDeliveryStatus::Skipped));
        });
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.async_bus.alert_delivery_query).hint_text("Search"),
    );
    if response.changed() {
        app.async_bus.alert_delivery_page = 0;
    }
    ui.separator();
}

fn filter_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    status: Option<AlertDeliveryStatus>,
) {
    if ui
        .selectable_label(app.async_bus.alert_delivery_status_filter == status, label)
        .clicked()
    {
        app.async_bus.alert_delivery_status_filter = status;
        app.async_bus.alert_delivery_page = 0;
    }
}

fn empty_title(filter_active: bool) -> &'static str {
    if filter_active {
        "No matching deliveries"
    } else {
        "No deliveries"
    }
}

fn empty_message(filter_active: bool) -> &'static str {
    if filter_active {
        "No recent records match the current filter."
    } else {
        "Enable routing and trigger a warning or critical event."
    }
}

fn alert_delivery_color(status: AlertDeliveryStatus) -> egui::Color32 {
    match status {
        AlertDeliveryStatus::Delivered => status_color(NodeStatus::Running),
        AlertDeliveryStatus::Failed => status_color(NodeStatus::Error),
        AlertDeliveryStatus::Skipped => muted_text(),
    }
}
