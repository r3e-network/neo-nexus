use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};

use crate::events::{RuntimeEvent, RuntimeEventFilter};

use super::{EventJournalReport, EventJournalReportExport, EventJournalReporter};

impl EventJournalReporter {
    pub fn write(
        output_dir: impl AsRef<Path>,
        database: impl AsRef<Path>,
        events: Vec<RuntimeEvent>,
        matched_event_count: usize,
        filter: &RuntimeEventFilter,
        application_version: impl Into<String>,
    ) -> Result<EventJournalReportExport> {
        Self::write_at(
            output_dir,
            database,
            events,
            matched_event_count,
            filter,
            application_version,
            current_unix_time()?,
        )
    }

    pub fn write_at(
        output_dir: impl AsRef<Path>,
        database: impl AsRef<Path>,
        events: Vec<RuntimeEvent>,
        matched_event_count: usize,
        filter: &RuntimeEventFilter,
        application_version: impl Into<String>,
        generated_at_unix: u64,
    ) -> Result<EventJournalReportExport> {
        let output_dir = output_dir.as_ref();
        fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "failed to create event journal report directory {}",
                output_dir.display()
            )
        })?;

        let report = EventJournalReport::from_events(
            database,
            events,
            matched_event_count,
            filter,
            application_version,
            generated_at_unix,
        );
        let (text_path, json_path) = write_report_files(output_dir, generated_at_unix, &report)?;

        Ok(EventJournalReportExport {
            output_dir: output_dir.to_path_buf(),
            text_path,
            json_path,
            report,
        })
    }
}

fn write_report_files(
    output_dir: &Path,
    generated_at_unix: u64,
    report: &EventJournalReport,
) -> Result<(PathBuf, PathBuf)> {
    let stem = format!("event-journal-{generated_at_unix}");
    let text_path = output_dir.join(format!("{stem}.txt"));
    let json_path = output_dir.join(format!("{stem}.json"));

    fs::write(&text_path, report.to_text()).with_context(|| {
        format!(
            "failed to write event journal report {}",
            text_path.display()
        )
    })?;
    fs::write(&json_path, report.to_json_text()?).with_context(|| {
        format!(
            "failed to write event journal report {}",
            json_path.display()
        )
    })?;

    Ok((text_path, json_path))
}

fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_secs())
}
