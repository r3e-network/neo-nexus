use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn export_support_bundle(&mut self) {
        match WorkspaceSupportBundleExporter::write(
            &self.repository,
            self.repository.db_path(),
            self.support_bundle_dir(),
            env!("CARGO_PKG_VERSION"),
        ) {
            Ok(export) => {
                let severity = support_bundle_event_severity(&export.status);
                let message = format!(
                    "Support bundle exported: {}, {} files, {} archive, {}",
                    export.status,
                    export.manifest.files.len(),
                    format_bytes(export.archive_bytes),
                    short_path(&export.archive_path, 48)
                );
                self.record_event(
                    None,
                    None,
                    EventKind::SupportBundleExported,
                    severity,
                    format!(
                        "Support bundle exported to {}; archive SHA-256 {}",
                        export.archive_path.display(),
                        export.archive_sha256
                    ),
                );
                self.notice = Some(message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}

fn support_bundle_event_severity(status: &str) -> EventSeverity {
    match status {
        "failed" | "blocked" => EventSeverity::Critical,
        "review" => EventSeverity::Warning,
        _ => EventSeverity::Info,
    }
}
