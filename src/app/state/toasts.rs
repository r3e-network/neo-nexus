//! Operator-facing toast stack. Notices still write through `NeoNexusApp::notice`
//! for test compatibility; each distinct notice is mirrored here so the status
//! bar and floating toast strip can show a short history with severity colour.

use std::time::{Duration, Instant};

use eframe::egui::{self, Color32};

use crate::app::theme;

const MAX_TOASTS: usize = 4;
const DEFAULT_TTL: Duration = Duration::from_secs(6);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    pub(in crate::app) fn color(self) -> Color32 {
        match self {
            Self::Info => theme::info(),
            Self::Success => theme::success(),
            Self::Warning => theme::warning(),
            Self::Error => theme::danger(),
        }
    }

    /// Infer severity from common notice phrasing so existing call sites keep
    /// writing plain strings while the toast strip still colour-codes failures.
    pub(in crate::app) fn infer(message: &str) -> Self {
        let lower = message.to_ascii_lowercase();
        if lower.contains("fail")
            || lower.contains("error")
            || lower.contains("not saved")
            || lower.contains("rejected")
            || lower.contains("invalid")
            || lower.contains("denied")
            || lower.contains("blocked")
        {
            Self::Error
        } else if lower.contains("warn") || lower.contains("before ") {
            Self::Warning
        } else if lower.contains("saved")
            || lower.contains("started")
            || lower.contains("stopped")
            || lower.contains("complete")
            || lower.contains("verified")
            || lower.contains("imported")
            || lower.contains("exported")
        {
            Self::Success
        } else {
            Self::Info
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::app) struct Toast {
    pub(in crate::app) message: String,
    pub(in crate::app) kind: ToastKind,
    pub(in crate::app) created_at: Instant,
    pub(in crate::app) sticky: bool,
}

#[derive(Debug, Default)]
pub(in crate::app) struct ToastStack {
    items: Vec<Toast>,
    /// Last `notice` value already mirrored, so repeated identical writes do not
    /// flood the stack every frame.
    last_mirrored: Option<String>,
}

impl ToastStack {
    pub(in crate::app) fn mirror_notice(&mut self, notice: Option<&str>) {
        match notice {
            None => {
                self.last_mirrored = None;
            }
            Some(message) if self.last_mirrored.as_deref() == Some(message) => {}
            Some(message) => {
                let kind = ToastKind::infer(message);
                let sticky = matches!(kind, ToastKind::Error);
                self.push(Toast {
                    message: message.to_string(),
                    kind,
                    created_at: Instant::now(),
                    sticky,
                });
                self.last_mirrored = Some(message.to_string());
            }
        }
    }

    pub(in crate::app) fn push(&mut self, toast: Toast) {
        self.items.push(toast);
        while self.items.len() > MAX_TOASTS {
            self.items.remove(0);
        }
    }

    pub(in crate::app) fn expire_due(&mut self) {
        let now = Instant::now();
        self.items
            .retain(|toast| toast.sticky || now.duration_since(toast.created_at) < DEFAULT_TTL);
    }

    pub(in crate::app) fn iter(&self) -> impl DoubleEndedIterator<Item = &Toast> + '_ {
        self.items.iter()
    }

    pub(in crate::app) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Compact toast strip for the status bar (latest only) and a small multi-toast
/// trail when several notices fire in sequence.
pub(in crate::app) fn render_toast_strip(ui: &mut egui::Ui, stack: &ToastStack) {
    if stack.is_empty() {
        ui.label(theme::muted_body("Ready"));
        return;
    }
    ui.horizontal(|ui| {
        for toast in stack.iter().rev().take(2) {
            let color = toast.kind.color();
            egui::Frame::new()
                .fill(Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 28))
                .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.55)))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(8, 2))
                .show(ui, |ui| {
                    ui.label(theme::body(truncate_notice(&toast.message, 56)).color(color));
                });
        }
    });
}

fn truncate_notice(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let mut out: String = text.chars().take(keep).collect();
    out.push('…');
    out
}
