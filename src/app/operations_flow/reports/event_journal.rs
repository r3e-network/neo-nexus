use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn export_event_journal_report(&mut self) {
        let filter = RuntimeEventFilter::new(
            self.event_severity_filter,
            self.event_query.clone(),
            DEFAULT_EVENT_EXPORT_LIMIT,
        );
        let matched_event_count = match self.repository.count_events(&filter) {
            Ok(count) => count,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };
        let events = match self.repository.list_events(filter.clone()) {
            Ok(events) => events,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };

        match EventJournalReporter::write(
            self.event_journal_export_dir(),
            self.repository.db_path(),
            events,
            matched_event_count,
            &filter,
            env!("CARGO_PKG_VERSION"),
        ) {
            Ok(export) => {
                let message = event_journal_notice(&filter, &export);
                self.record_event(
                    None,
                    None,
                    EventKind::EventJournalExported,
                    EventSeverity::Info,
                    format!(
                        "Event journal exported to {}; JSON {}",
                        export.text_path.display(),
                        export.json_path.display()
                    ),
                );
                self.notice = Some(message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}

fn event_journal_notice(filter: &RuntimeEventFilter, export: &EventJournalReportExport) -> String {
    let severity_label = filter
        .severity
        .map_or_else(|| "all".to_string(), |severity| severity.to_string());
    let query_label = if filter.query.trim().is_empty() {
        "-".to_string()
    } else {
        filter.query.trim().to_string()
    };
    format!(
        "Event journal exported: {} of {} events, severity {}, query {}, {}",
        export.report.exported_event_count,
        export.report.matched_event_count,
        severity_label,
        query_label,
        short_path(&export.text_path, 48)
    )
}
