use eframe::egui;

pub(super) const PANEL_GAP: f32 = 8.0;

const TOP_MIN_HEIGHT: f32 = 210.0;
const TOP_MAX_HEIGHT: f32 = 300.0;
const BOTTOM_MIN_HEIGHT: f32 = 308.0;
const READINESS_MIN_WIDTH: f32 = 300.0;
const READINESS_MAX_WIDTH: f32 = 520.0;
const ACTION_MIN_WIDTH: f32 = 300.0;
const MATRIX_MIN_WIDTH: f32 = 340.0;
const MATRIX_MAX_WIDTH: f32 = 760.0;
const SIDE_MIN_WIDTH: f32 = 220.0;
const SAFETY_MIN_HEIGHT: f32 = 164.0;
const SAFETY_MAX_HEIGHT: f32 = 196.0;
const JOURNAL_MIN_HEIGHT: f32 = 136.0;

pub(super) struct OperationsLayout {
    pub(super) top_height: f32,
    pub(super) readiness_width: f32,
    pub(super) action_width: f32,
}

pub(super) struct OperationsBottomLayout {
    pub(super) matrix_width: f32,
    pub(super) side_width: f32,
    pub(super) height: f32,
}

pub(super) struct OperationsSideLayout {
    pub(super) width: f32,
    pub(super) safety_height: f32,
    pub(super) journal_height: f32,
}

pub(super) fn operations_layout(available: egui::Vec2) -> OperationsLayout {
    let top_height = (available.y * 0.52)
        .clamp(TOP_MIN_HEIGHT, TOP_MAX_HEIGHT)
        .min((available.y - PANEL_GAP - BOTTOM_MIN_HEIGHT).max(TOP_MIN_HEIGHT));
    let readiness_width = (available.x * 0.46)
        .clamp(READINESS_MIN_WIDTH, READINESS_MAX_WIDTH)
        .min((available.x - PANEL_GAP - ACTION_MIN_WIDTH).max(READINESS_MIN_WIDTH));
    let action_width = (available.x - readiness_width - PANEL_GAP).max(ACTION_MIN_WIDTH);

    OperationsLayout {
        top_height,
        readiness_width,
        action_width,
    }
}

pub(super) fn operations_bottom_layout(available: egui::Vec2) -> OperationsBottomLayout {
    let matrix_width = (available.x * 0.64)
        .clamp(MATRIX_MIN_WIDTH, MATRIX_MAX_WIDTH)
        .min((available.x - PANEL_GAP - SIDE_MIN_WIDTH).max(MATRIX_MIN_WIDTH));
    let side_width = (available.x - matrix_width - PANEL_GAP).max(SIDE_MIN_WIDTH);

    OperationsBottomLayout {
        matrix_width,
        side_width,
        height: available.y,
    }
}

pub(super) fn operations_side_layout(available: egui::Vec2) -> OperationsSideLayout {
    let safety_height = (available.y * 0.54)
        .clamp(SAFETY_MIN_HEIGHT, SAFETY_MAX_HEIGHT)
        .min((available.y - JOURNAL_MIN_HEIGHT).max(SAFETY_MIN_HEIGHT));
    let journal_height = (available.y - safety_height - PANEL_GAP).max(JOURNAL_MIN_HEIGHT);

    OperationsSideLayout {
        width: available.x,
        safety_height,
        journal_height,
    }
}
