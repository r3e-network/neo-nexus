use std::path::Path;

use eframe::egui;

use crate::app::{
    domain::{EventSeverity, WorkspaceBackupValidation},
    theme::muted_text,
};

use super::super::super::helpers::{event_color, score_color};

pub(in crate::app) struct BackupValidationSummary {
    pub(in crate::app) status_label: String,
    pub(in crate::app) counts_label: String,
    pub(in crate::app) color: egui::Color32,
    pub(in crate::app) hint: String,
    pub(super) is_current: bool,
}

impl BackupValidationSummary {
    pub(super) fn from_validation(
        latest_backup: Option<&Path>,
        validation: Option<&WorkspaceBackupValidation>,
    ) -> Self {
        match (latest_backup, validation) {
            (None, _) => Self::none_available(),
            (Some(_), None) => Self::not_checked(),
            (Some(latest), Some(validation)) if validation_source_matches(latest, validation) => {
                Self::checked(validation)
            }
            (Some(_), Some(validation)) => Self::stale(validation),
        }
    }

    fn checked(validation: &WorkspaceBackupValidation) -> Self {
        Self {
            status_label: "validated".to_string(),
            counts_label: backup_validation_counts(validation),
            color: score_color(100),
            hint: "Latest backup validation passed.".to_string(),
            is_current: true,
        }
    }

    fn stale(validation: &WorkspaceBackupValidation) -> Self {
        Self {
            status_label: "stale".to_string(),
            counts_label: backup_validation_counts(validation),
            color: event_color(EventSeverity::Warning),
            hint: "Validation belongs to a different backup; validate latest before import."
                .to_string(),
            is_current: false,
        }
    }

    fn not_checked() -> Self {
        Self {
            status_label: "not validated".to_string(),
            counts_label: "run validate".to_string(),
            color: muted_text(),
            hint: "Validate latest backup before import.".to_string(),
            is_current: false,
        }
    }

    fn none_available() -> Self {
        Self {
            status_label: "no backup".to_string(),
            counts_label: "-".to_string(),
            color: muted_text(),
            hint: "Export or place a backup before validation.".to_string(),
            is_current: false,
        }
    }
}

fn validation_source_matches(latest: &Path, validation: &WorkspaceBackupValidation) -> bool {
    validation
        .source_path
        .as_deref()
        .is_some_and(|source| source == latest)
}

fn backup_validation_counts(validation: &WorkspaceBackupValidation) -> String {
    format!(
        "{} nodes / {} events",
        validation.node_count, validation.event_count
    )
}
