use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn watchdog_label(&self, node_id: &str) -> String {
        match self.watchdog.status(node_id, Instant::now()) {
            WatchdogStatus::Idle => "idle".to_string(),
            WatchdogStatus::Pending { attempt, remaining } => {
                format!("attempt {attempt} in {}", format_duration(remaining))
            }
            WatchdogStatus::Exhausted { attempts } => {
                format!("exhausted after {attempts}")
            }
        }
    }
}
