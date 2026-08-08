#[cfg(unix)]
use std::{
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use super::NeoNexusApp;

/// Drives the app's process reconciliation until a predicate holds.
///
/// Only the sidecar watchdog suite uses it, and that suite needs a real forkable
/// process, so it is `#[cfg(unix)]` — and so is this.
#[cfg(unix)]
pub(super) fn reconcile_app_processes_until(
    app: &mut NeoNexusApp,
    timeout: Duration,
    mut condition: impl FnMut(&NeoNexusApp) -> bool,
) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        app.reconcile_supervised_processes();
        if condition(app) {
            return true;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        thread::sleep(Duration::from_millis(25).min(remaining));
    }
    app.reconcile_supervised_processes();
    condition(app)
}
