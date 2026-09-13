//! Cursor management for incremental log collection.
//!
//! Each cursor identifies a precise reading position: (node_id, file_generation, byte_offset).
//! File rotation/truncation/deletion/restart events invalidate the generation token,
//! causing the offset to reset and trigger tail-first baseline reads.
//!
//! The buffer budget limits are enforced globally (4MiB + 8192 lines) with per-node
//! allocations (256KiB max), using round-robin scheduling for fair sharing.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Duration;

use crate::types::NodeConfig;

// ============================================================================
// Constants
// ============================================================================

/// Maximum bytes retained in memory for a single node's pending batch
pub(crate) const NODE_BUFFER_LIMIT: usize = 256 * 1024; // 256 KiB

/// Global aggregate budget for all nodes' pending batches
pub(crate) const GLOBAL_BUFFER_LIMIT: usize = 4 * 1024 * 1024; // 4 MiB

/// Maximum total lines across all nodes before oldest entries are evicted
pub(crate) const GLOBAL_LINE_LIMIT: usize = 8192;

/// Maximum single line length before truncation occurs
pub(crate) const MAX_LINE_LENGTH: usize = 64 * 1024; // 64 KiB

/// Baseline read window size for first-time tails
pub(crate) const BASELINE_READ_WINDOW: usize = 64 * 1024; // 64 KiB

/// Interval between cursor checkpoint commits
pub(crate) const CURSOR_CHECKPOINT_INTERVAL: Duration = Duration::from_secs(30);

// ============================================================================
// Core Data Types
// ============================================================================

/// File identity tracking rotation/generation changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct FileIdentity {
    /// Generation counter incremented on file rotation/truncation
    pub generation: u64,
    /// Inode/device identifier (cross-platform fallback to path hash when unavailable)
    pub device_id: u64,
    /// Last known file size at last healthy snapshot
    pub file_size: u64,
}

impl FileIdentity {
    pub fn new(generation: u64, device_id: u64, file_size: u64) -> Self {
        Self {
            generation,
            device_id,
            file_size,
        }
    }

    /// Increment generation to signal rotation/truncation event
    pub fn rotate(&self) -> Self {
        Self {
            generation: self.generation.wrapping_add(1),
            device_id: self.device_id,
            file_size: 0, // Will be updated by next snapshot
        }
    }
}

/// Read cursor tracking precise byte-level position within a log file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LogCursor {
    /// Node ID owning this cursor
    pub node_id: String,
    /// Current file identity – invalidates on rotation/truncation/deletion
    pub file_identity: FileIdentity,
    /// Byte offset from start of current generation
    pub byte_offset: u64,
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,
    /// Last update timestamp (Unix epoch seconds)
    pub updated_at: u64,
}

impl LogCursor {
    pub fn new(node_id: impl Into<String>, file_identity: FileIdentity) -> Self {
        let now = unix_timestamp();
        Self {
            node_id: node_id.into(),
            file_identity,
            byte_offset: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn advance(&mut self, bytes: u64) {
        self.byte_offset = self.byte_offset.saturating_add(bytes);
        self.updated_at = unix_timestamp();
    }

    pub fn reset(&mut self, new_identity: FileIdentity) {
        self.file_identity = new_identity;
        self.byte_offset = 0;
        self.created_at = self.updated_at;
        self.updated_at = unix_timestamp();
    }

    /// Bind the cursor to the current file identity without resetting the byte offset.
    /// Used after a baseline read establishes the current generation so that the next
    /// round is treated as incremental rather than baseline.
    pub fn bind_identity(&mut self, identity: FileIdentity) {
        self.file_identity = identity;
        self.updated_at = unix_timestamp();
    }

    pub fn is_stale(&self, age_seconds: u64) -> bool {
        unix_timestamp().saturating_sub(self.updated_at) > age_seconds
    }
}

// ============================================================================
// Cursor Store & Budget Manager
// ============================================================================

/// Centralized store managing cursors per-node with global budget enforcement.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CursorStore {
    cursors: std::collections::HashMap<String, LogCursor>,
    /// Round-robin queue of active node IDs
    round_robin_queue: VecDeque<String>,
    /// Pending bytes counter (for budget enforcement)
    pending_bytes: usize,
    /// Pending lines counter (for budget enforcement)
    pending_lines: usize,
}

impl CursorStore {
    pub fn new() -> Self {
        Self {
            cursors: std::collections::HashMap::new(),
            round_robin_queue: VecDeque::new(),
            pending_bytes: 0,
            pending_lines: 0,
        }
    }

    /// Get or create cursor for a node
    pub fn get_or_create(&mut self, node: &NodeConfig) -> &mut LogCursor {
        self.round_robin_queue.push_back(node.id.clone());
        self.cursors
            .entry(node.id.clone())
            .or_insert_with(|| LogCursor::new(&node.id, FileIdentity::default()))
    }

    /// Check if adding `extra_bytes` would exceed budget
    pub fn can_accept(&self, extra_bytes: usize, extra_lines: usize) -> bool {
        self.pending_bytes.saturating_add(extra_bytes) <= GLOBAL_BUFFER_LIMIT
            && self.pending_lines.saturating_add(extra_lines) <= GLOBAL_LINE_LIMIT
    }

    /// Acquire budget quota, returning actual allowance
    pub fn acquire_budget(
        &mut self,
        requested_bytes: usize,
        requested_lines: usize,
    ) -> (usize, usize) {
        let allowed_bytes = GLOBAL_BUFFER_LIMIT
            .saturating_sub(self.pending_bytes)
            .min(requested_bytes)
            .min(NODE_BUFFER_LIMIT);
        let allowed_lines = GLOBAL_LINE_LIMIT
            .saturating_sub(self.pending_lines)
            .min(requested_lines)
            .min(MAX_LINE_LENGTH);

        if allowed_bytes > 0 || allowed_lines > 0 {
            self.pending_bytes = self.pending_bytes.saturating_add(allowed_bytes);
            self.pending_lines = self.pending_lines.saturating_add(allowed_lines);
        }

        (allowed_bytes, allowed_lines)
    }

    /// Release budget after successful commit
    pub fn release_budget(&mut self, bytes: usize, lines: usize) {
        self.pending_bytes = self.pending_bytes.saturating_sub(bytes);
        self.pending_lines = self.pending_lines.saturating_sub(lines);

        // Prune stale cursors when under pressure
        if self.pending_bytes == 0 && self.pending_lines == 0 {
            self.prune_stale(CURSOR_CHECKPOINT_INTERVAL.as_secs());
        }
    }

    /// Remove cursors older than `age_seconds` from inactive nodes
    pub fn prune_stale(&mut self, age_seconds: u64) {
        self.cursors
            .retain(|_, cursor| !cursor.is_stale(age_seconds));
        self.round_robin_queue
            .retain(|id| self.cursors.contains_key(id));
    }

    /// Detect file rotation/truncation/deletion events
    pub fn detect_file_change(
        &mut self,
        node_id: &str,
        current_path: &Path,
    ) -> Option<FileIdentity> {
        let cursor = self.cursors.get(node_id)?;
        let current = file_identity_from_path(current_path)?;

        // Generation mismatch or inode change indicates rotation/truncation
        if cursor.file_identity.generation != current.generation
            || cursor.file_identity.device_id != current.device_id
        {
            // File was rotated/truncated/deleted - reset cursor
            let new_identity = current.rotate();

            // Update cursor with new generation
            if let Some(c) = self.cursors.get_mut(node_id) {
                c.reset(new_identity);
            }

            return Some(new_identity);
        }

        // Size decrease indicates truncation
        if current.file_size < cursor.file_identity.file_size {
            let new_identity = current.rotate();

            if let Some(c) = self.cursors.get_mut(node_id) {
                c.reset(new_identity);
            }

            return Some(new_identity);
        }

        None
    }

    /// Get mutable reference to cursor
    pub fn get_cursor_mut(&mut self, node_id: &str) -> Option<&mut LogCursor> {
        self.cursors.get_mut(node_id)
    }

    /// Get immutable reference to cursor
    pub fn get_cursor(&self, node_id: &str) -> Option<&LogCursor> {
        self.cursors.get(node_id)
    }

    /// Iterator over all active cursors.
    ///
    /// Reserved inspection/persistence surface: the collection worker holds its
    /// `CursorStore` thread-locally and drives reads through the offset/identity/
    /// budget path above, so these helpers have no live caller today. They are
    /// kept as the checkpoint-persistence API (snapshot, restore, reset) rather
    /// than removed, and are exercised by the cursor unit tests.
    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = (&String, &LogCursor)> {
        self.cursors.iter()
    }

    /// Clear all cursors (for clean restart)
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.cursors.clear();
        self.round_robin_queue.clear();
        self.pending_bytes = 0;
        self.pending_lines = 0;
    }

    /// Load cursors from persistence layer
    #[allow(dead_code)]
    pub fn load_from_json(data: &[u8]) -> Result<Self> {
        let store: CursorStore =
            serde_json::from_slice(data).context("failed to deserialize cursor store")?;
        Ok(store)
    }

    /// Serialize cursors to JSON for persistence
    #[allow(dead_code)]
    pub fn to_json(&self) -> Result<Vec<u8>> {
        let data = serde_json::to_vec(self).context("failed to serialize cursor store")?;
        Ok(data)
    }
}

/// Compute file identity from filesystem metadata without requiring std::os traits
pub(crate) fn file_identity_from_path(path: &Path) -> Option<FileIdentity> {
    use std::fs;

    let metadata = fs::metadata(path).ok()?;
    let file_size = metadata.len();

    // Windows fallback: use modification time as generation proxy
    // On Unix, we could use inode numbers here, but we avoid platform-specific imports
    let modified = metadata.modified().ok()?;
    let epoch = modified
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    // Simple hash-based device ID for cross-platform uniqueness
    let path_hash = crc32fast::hash(path.to_string_lossy().as_ref());

    Some(FileIdentity::new(epoch / 3600, path_hash as u64, file_size))
}

/// CRC32 checksum for simple hashing (Windows compatible)
mod crc32fast {
    pub fn hash(data: &str) -> u32 {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(data.as_bytes());
        hasher.finalize()
    }
}

/// Current Unix timestamp in seconds
fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

// Unit tests live under tests/ but are compiled as a child of this module so
// they can assert on the store's private budget counters and queue.
#[cfg(test)]
#[path = "../../tests/unit/logs/cursor_tests.rs"]
mod cursor_tests;
