use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn refresh_metrics_now(&mut self) {
        self.metrics_snapshot = self.metrics.refresh(&self.nodes, Instant::now());
    }

    pub(in crate::app) fn refresh_metrics_if_due(&mut self) {
        if let Some(snapshot) = self.metrics.refresh_if_due(&self.nodes, Instant::now()) {
            self.metrics_snapshot = snapshot;
        }
    }
}
