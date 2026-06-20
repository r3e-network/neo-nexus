use super::super::super::*;

pub(super) fn recover_transient_runtime_state(app: &mut NeoNexusApp) {
    match app.repository.clear_transient_runtime_state() {
        Ok(count) if count > 0 => record_runtime_recovery(app, count),
        Ok(_) => {}
        Err(error) => app.notice = Some(error.to_string()),
    }
}

fn record_runtime_recovery(app: &mut NeoNexusApp, count: usize) {
    let message = format!("Recovered {count} stale runtime state records");
    app.notice = Some(message.clone());
    let _ = app.repository.record_event(NewRuntimeEvent {
        node_id: None,
        node_name: None,
        kind: EventKind::RuntimeRecovered,
        severity: EventSeverity::Warning,
        message,
    });
}
