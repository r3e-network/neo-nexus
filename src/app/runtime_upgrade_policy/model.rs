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
    breakdown: RuntimeUpgradePolicyBreakdown,
    catalog_label: String,
    limited: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RuntimeUpgradePolicyBreakdown {
    pub(super) stopped_ready: usize,
    pub(super) running_ready: usize,
    pub(super) planned_stopped: usize,
    pub(super) planned_running: usize,
    pub(super) blocked_active: usize,
    pub(super) current_or_unavailable: usize,
}

impl RuntimeUpgradePolicyBreakdown {
    fn ready(self) -> usize {
        self.stopped_ready + self.running_ready
    }

    fn planned(self) -> usize {
        self.planned_stopped + self.planned_running
    }
}

impl RuntimeUpgradePolicySummary {
    pub(super) fn new(
        upgraded: usize,
        breakdown: RuntimeUpgradePolicyBreakdown,
        catalog_label: String,
        limited: bool,
    ) -> Self {
        Self {
            upgraded,
            breakdown,
            catalog_label,
            limited,
        }
    }

    pub(super) fn message(&self, mode: RuntimeUpgradeRunMode) -> String {
        let limited = if self.limited { "; batch limited" } else { "" };
        let breakdown = self.breakdown;
        format!(
            "Runtime upgrade policy {} via {}: {} upgraded, {} ready ({} stopped, {} running), planned {} ({} stopped, {} running), {} blocked, {} current/unavailable{}",
            mode.label(),
            self.catalog_label,
            self.upgraded,
            breakdown.ready(),
            breakdown.stopped_ready,
            breakdown.running_ready,
            breakdown.planned(),
            breakdown.planned_stopped,
            breakdown.planned_running,
            breakdown.blocked_active,
            breakdown.current_or_unavailable,
            limited
        )
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app/runtime_upgrade_policy/model/tests.rs"]
mod tests;
