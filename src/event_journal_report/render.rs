use anyhow::Result;

use super::{EventJournalReport, EventJournalReportExport};

impl EventJournalReportExport {
    pub fn to_cli_text(&self) -> String {
        format!(
            "event-journal-report: ok\nfilter-severity: {severity}\nfilter-query: {query}\nmatched-events: {matched}\nexported-events: {exported}\nreport-text: {text}\nreport-json: {json}\n",
            severity = self.report.filter.severity.as_deref().unwrap_or("all"),
            query = non_empty_or_dash(&self.report.filter.query),
            matched = self.report.matched_event_count,
            exported = self.report.exported_event_count,
            text = self.text_path.display(),
            json = self.json_path.display(),
        )
    }
}

impl EventJournalReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            "event-journal-report: ok".to_string(),
            format!("application-version: {}", self.application_version),
            format!("generated-at-unix: {}", self.generated_at_unix),
            format!("database: {}", self.database),
            format!("requested-limit: {}", self.requested_limit),
            format!(
                "filter-severity: {}",
                self.filter.severity.as_deref().unwrap_or("all")
            ),
            format!("filter-query: {}", non_empty_or_dash(&self.filter.query)),
            format!("matched-events: {}", self.matched_event_count),
            format!("exported-events: {}", self.exported_event_count),
        ];

        if self.events.is_empty() {
            lines.push("event: none".to_string());
        } else {
            for event in &self.events {
                lines.push(format!(
                    "event: {} | {} | {} | {} | {}",
                    event.occurred_at_unix,
                    event.severity,
                    event.kind,
                    event.node_name.as_deref().unwrap_or("-"),
                    event.message
                ));
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }

    pub fn to_json_text(&self) -> Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(self)?))
    }
}

fn non_empty_or_dash(value: &str) -> &str {
    if value.is_empty() {
        "-"
    } else {
        value
    }
}
