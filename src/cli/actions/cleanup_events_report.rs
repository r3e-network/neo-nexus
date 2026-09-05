//! Event journal archival command.
//!
//! This module is now integrated into repository::events_health::events::prune
//! as public methods on Repository. The CLI entry point remains here.

use crate::core::workspace::Repository;
use std::path::PathBuf;
use anyhow::Result;

/// Export old events before purging them.
pub fn export_events_before(
    repository: &Repository,
    max_age_days: u64,
    output_path: PathBuf,
) -> Result<usize> {
    // Load events older than cutoff from repository module
    repository.export_events_before(max_age_days, output_path)
}

/// Purge old events after export. Returns number of rows deleted.
pub fn purge_old_events(repository: &Repository, max_age_days: u64) -> Result<usize> {
    repository.purge_events_before(max_age_days)
}
