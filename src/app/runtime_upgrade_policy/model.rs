#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RuntimeUpgradeRunMode {
    Manual,
    Scheduled,
}

impl RuntimeUpgradeRunMode {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Manual => "manual run",
            Self::Scheduled => "scheduled run",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RuntimeUpgradePolicySummary {
    pub(super) upgraded: usize,
    available: usize,
    blocked_running: usize,
    current_or_unavailable: usize,
    catalog_label: String,
    limited: bool,
}

impl RuntimeUpgradePolicySummary {
    pub(super) fn new(
        upgraded: usize,
        available: usize,
        blocked_running: usize,
        current_or_unavailable: usize,
        catalog_label: String,
        limited: bool,
    ) -> Self {
        Self {
            upgraded,
            available,
            blocked_running,
            current_or_unavailable,
            catalog_label,
            limited,
        }
    }

    pub(super) fn message(&self, mode: RuntimeUpgradeRunMode) -> String {
        let limited = if self.limited { "; batch limited" } else { "" };
        format!(
            "Runtime upgrade policy {} via {}: {} upgraded, {} ready, {} blocked, {} current/unavailable{}",
            mode.label(),
            self.catalog_label,
            self.upgraded,
            self.available,
            self.blocked_running,
            self.current_or_unavailable,
            limited
        )
    }
}
