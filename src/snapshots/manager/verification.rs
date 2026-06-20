use anyhow::Result;

use super::FastSyncSnapshotManager;
use crate::snapshots::{
    current_unix_time,
    io::sha256_file,
    model::{FastSyncSnapshot, SnapshotVerification},
    validation::{normalize_sha256, verified_source_path},
};

impl FastSyncSnapshotManager {
    pub fn verify(snapshot: &FastSyncSnapshot) -> Result<SnapshotVerification> {
        let source = verified_source_path(&snapshot.source_path)?;
        let (sha256, bytes) = sha256_file(&source)?;
        let expected_sha256 = normalize_sha256(&snapshot.expected_sha256)?;
        Ok(SnapshotVerification {
            matches: sha256 == expected_sha256,
            sha256,
            expected_sha256,
            bytes,
            verified_at_unix: current_unix_time()?,
        })
    }
}
