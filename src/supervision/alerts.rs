//! Handing new journal entries to the alert route the Alerts page configures.

use crate::alerts::{deliver_webhook_alert, should_route_alert};

use super::state::{EngineState, LoopState};

/// How many journal entries are offered to the route per tick. One delivery per
/// tick: a webhook that is down should not hold up the rest of the loop, and the
/// journal keeps the backlog visible.
const JOURNAL_SCAN_LIMIT: usize = 25;
const ALERT_DELIVERY_RETAIN: usize = 50;

impl LoopState {
    /// Offer anything new since the last scan to the configured alert route.
    ///
    /// Forward pagination (`list_events_after`) is used rather than "latest N":
    /// during a burst that produces more than the scan limit between ticks, a
    /// descending "latest 25" window would let entries older than the window but
    /// newer than the last routed id slip past unrouted. Reading strictly after
    /// the checkpoint in ascending id order guarantees no unseen entry is skipped.
    pub(super) fn route_alerts(&mut self, state: &EngineState) {
        let Ok(policy) = state.repository.load_alert_routing_policy() else {
            return;
        };
        let Ok(events) = state
            .repository
            .list_events_after(self.last_routed_event, JOURNAL_SCAN_LIMIT)
        else {
            return;
        };
        // Ascending id order: the first entry is the earliest unrouted one.
        let Some(event) = events.into_iter().next() else {
            return;
        };

        if !should_route_alert(&policy, &event) {
            self.last_routed_event = event.id;
            return;
        }
        self.last_routed_event = event.id;

        let report = deliver_webhook_alert(&policy, &event, env!("CARGO_PKG_VERSION"));
        if state.repository.record_alert_delivery(&report).is_err() {
            return;
        }
        let _ = state
            .repository
            .prune_alert_deliveries_keep_recent(ALERT_DELIVERY_RETAIN);
        // A failed delivery is recorded in the deliveries table, which the
        // Alerts page already renders; the journal is for state changes.
        let _ = report.status;
    }
}
