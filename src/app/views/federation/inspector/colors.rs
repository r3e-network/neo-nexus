use eframe::egui;

use crate::{app::theme::status_color, federation::RemoteProbeStatus, types::NodeStatus};

pub(super) fn remote_enabled_color(enabled: bool) -> egui::Color32 {
    if enabled {
        status_color(NodeStatus::Running)
    } else {
        crate::app::theme::muted_text()
    }
}

pub(super) fn remote_probe_color(status: RemoteProbeStatus) -> egui::Color32 {
    match status {
        RemoteProbeStatus::Healthy => status_color(NodeStatus::Running),
        RemoteProbeStatus::Degraded | RemoteProbeStatus::Disabled => {
            egui::Color32::from_rgb(202, 138, 4)
        }
        RemoteProbeStatus::Unreachable => status_color(NodeStatus::Error),
    }
}
