use eframe::egui;

use crate::{
    app::theme::{accent, muted_text},
    types::NodeStatus,
};

pub(super) fn catalog_upgrade_state(
    status: NodeStatus,
    has_catalog: bool,
    has_plan: bool,
) -> &'static str {
    if !status.is_stopped() {
        "stop node first"
    } else if !has_catalog {
        "load catalog"
    } else if !has_plan {
        "catalog current"
    } else {
        "catalog upgrade ready"
    }
}

pub(super) fn catalog_upgrade_color(
    status: NodeStatus,
    has_catalog: bool,
    has_plan: bool,
) -> egui::Color32 {
    if status.is_stopped() && has_catalog && has_plan {
        accent()
    } else {
        muted_text()
    }
}
