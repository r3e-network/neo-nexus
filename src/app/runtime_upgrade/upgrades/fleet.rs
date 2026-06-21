use crate::app::domain::{EventKind, EventSeverity, NodeStatus};

use super::super::super::NeoNexusApp;

impl NeoNexusApp {
    pub(in crate::app) fn upgrade_fleet_nodes_from_catalog(&mut self) {
        let Some(fleet_plan) = self.catalog_fleet_upgrade_plan() else {
            let message = "Load a runtime catalog before running fleet upgrades".to_string();
            self.record_fleet_upgrade_run(EventSeverity::Warning, message.clone());
            self.notice = Some(message);
            return;
        };
        let mut breakdown = FleetUpgradeBreakdown::new(
            fleet_plan.blocked_active,
            fleet_plan.current_or_unavailable,
        );
        let candidates = fleet_plan.into_ready_candidates();
        if candidates.is_empty() {
            let message = format!(
                "No fleet nodes have newer compatible catalog releases ({})",
                breakdown.label()
            );
            self.record_fleet_upgrade_run(EventSeverity::Info, message.clone());
            self.notice = Some(message);
            return;
        }

        let mut upgraded = 0usize;
        let mut last_message = String::new();
        for plan in candidates {
            let Some(node) = self
                .nodes
                .iter()
                .find(|node| node.id == plan.node_id)
                .cloned()
            else {
                continue;
            };
            match self.apply_catalog_upgrade_plan_to_node(&node, &plan) {
                Ok(message) => {
                    upgraded += 1;
                    breakdown.record_applied(node.status);
                    last_message = message;
                }
                Err(error) => {
                    let message = format!(
                        "Fleet catalog upgrade stopped after {upgraded} {} while upgrading {} ({}): {error}",
                        node_count_label(upgraded),
                        node.name,
                        breakdown.label()
                    );
                    self.record_fleet_upgrade_run(EventSeverity::Warning, message.clone());
                    self.reload_nodes();
                    self.notice = Some(message);
                    return;
                }
            }
        }

        self.reload_nodes();
        let message = fleet_upgrade_notice(upgraded, &last_message, breakdown);
        self.record_fleet_upgrade_run(EventSeverity::Info, message.clone());
        self.notice = Some(message);
    }

    fn record_fleet_upgrade_run(&mut self, severity: EventSeverity, message: String) {
        self.record_event(
            None,
            None,
            EventKind::RuntimeFleetUpgradeRun,
            severity,
            message,
        );
    }
}

#[derive(Debug, Clone, Copy)]
struct FleetUpgradeBreakdown {
    stopped_direct: usize,
    running_rollout: usize,
    blocked_active: usize,
    current_or_unavailable: usize,
}

impl FleetUpgradeBreakdown {
    fn new(blocked_active: usize, current_or_unavailable: usize) -> Self {
        Self {
            stopped_direct: 0,
            running_rollout: 0,
            blocked_active,
            current_or_unavailable,
        }
    }

    fn record_applied(&mut self, status: NodeStatus) {
        match status {
            NodeStatus::Stopped => self.stopped_direct += 1,
            NodeStatus::Running => self.running_rollout += 1,
            _ => {}
        }
    }

    fn label(self) -> String {
        format!(
            "{} stopped direct, {} running rollout, {} blocked active, {} current/unavailable",
            self.stopped_direct,
            self.running_rollout,
            self.blocked_active,
            self.current_or_unavailable
        )
    }
}

fn fleet_upgrade_notice(
    upgraded: usize,
    last_message: &str,
    breakdown: FleetUpgradeBreakdown,
) -> String {
    if upgraded == 1 {
        format!("{} ({})", last_message, breakdown.label())
    } else {
        format!(
            "Fleet catalog upgrade applied to {upgraded} nodes ({})",
            breakdown.label()
        )
    }
}

fn node_count_label(count: usize) -> &'static str {
    if count == 1 {
        "node"
    } else {
        "nodes"
    }
}
