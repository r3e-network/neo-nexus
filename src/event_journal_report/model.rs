use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{
    events::{RuntimeEvent, RuntimeEventFilter},
    redaction::redact_sensitive_text,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventJournalReportExport {
    pub output_dir: PathBuf,
    pub text_path: PathBuf,
    pub json_path: PathBuf,
    pub report: EventJournalReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventJournalReport {
    pub schema_version: u32,
    pub application: &'static str,
    pub application_version: String,
    pub generated_at_unix: u64,
    pub database: String,
    pub requested_limit: usize,
    pub filter: EventJournalReportFilter,
    pub matched_event_count: usize,
    pub exported_event_count: usize,
    pub events: Vec<EventJournalEventReport>,
}

impl EventJournalReport {
    pub fn from_events(
        database: impl AsRef<Path>,
        events: Vec<RuntimeEvent>,
        matched_event_count: usize,
        filter: &RuntimeEventFilter,
        application_version: impl Into<String>,
        generated_at_unix: u64,
    ) -> Self {
        let exported_event_count = events.len();
        let events = events
            .into_iter()
            .map(|event| EventJournalEventReport {
                id: event.id,
                occurred_at_unix: event.occurred_at_unix,
                node_id: event.node_id,
                node_name: event.node_name,
                kind: event.kind.to_string(),
                severity: event.severity.to_string(),
                message: redact_sensitive_text(&event.message),
            })
            .collect();

        Self {
            schema_version: 1,
            application: "NeoNexus",
            application_version: application_version.into(),
            generated_at_unix,
            database: database.as_ref().display().to_string(),
            requested_limit: filter.limit,
            filter: EventJournalReportFilter::from_filter(filter),
            matched_event_count,
            exported_event_count,
            events,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventJournalReportFilter {
    pub severity: Option<String>,
    pub query: String,
}

impl EventJournalReportFilter {
    fn from_filter(filter: &RuntimeEventFilter) -> Self {
        Self {
            severity: filter.severity.map(|severity| severity.to_string()),
            query: filter.query.trim().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventJournalEventReport {
    pub id: i64,
    pub occurred_at_unix: u64,
    pub node_id: Option<String>,
    pub node_name: Option<String>,
    pub kind: String,
    pub severity: String,
    pub message: String,
}
