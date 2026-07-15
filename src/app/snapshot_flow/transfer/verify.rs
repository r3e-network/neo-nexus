use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn verify_selected_snapshot(&mut self) {
        let Some(snapshot) = self.selected_fast_sync_snapshot() else {
            self.session.notice = Some("Select a fast sync snapshot before verification".to_string());
            return;
        };

        match FastSyncSnapshotManager::verify(&snapshot) {
            Ok(verification) => {
                let matches = verification.matches;
                if let Err(error) = self
                    .repository
                    .mark_fast_sync_snapshot_verified(&snapshot.id, &verification)
                {
                    self.session.notice = Some(error.to_string());
                    return;
                }

                let message = if matches {
                    format!(
                        "{} verified: {} bytes, SHA-256 matched",
                        snapshot.label, verification.bytes
                    )
                } else {
                    format!(
                        "{} verification mismatch: expected {}, got {}",
                        snapshot.label, verification.expected_sha256, verification.sha256
                    )
                };
                self.record_event(
                    None,
                    None,
                    EventKind::SnapshotVerified,
                    if matches {
                        EventSeverity::Info
                    } else {
                        EventSeverity::Warning
                    },
                    message.clone(),
                );
                self.session.notice = Some(message);
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
