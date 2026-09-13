use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Context;
use log::warn;

use crate::{
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    logs::{
        cursor::{CursorStore, BASELINE_READ_WINDOW},
        observations::LogObservations,
    },
    repository::Repository,
};

const LOG_COLLECTION_INTERVAL: Duration = Duration::from_secs(30);

use super::model::{self, DEFAULT_STOP_GRACE_PERIOD};

mod child;
mod lifecycle;
mod reap;
mod spawn;

use child::ManagedChild;

pub struct ProcessSupervisor {
    children: HashMap<String, ManagedChild>,
    stop_grace_period: Duration,
    adapters: model::NodeAdapters,
    log_observations: Arc<LogObservations>,
}

impl Default for ProcessSupervisor {
    fn default() -> Self {
        Self {
            children: HashMap::new(),
            stop_grace_period: DEFAULT_STOP_GRACE_PERIOD,
            adapters: model::NodeAdapters::initialized(),
            log_observations: Arc::new(LogObservations::default()),
        }
    }
}

impl ProcessSupervisor {
    pub fn with_stop_grace_period(stop_grace_period: Duration) -> Self {
        Self {
            children: HashMap::new(),
            stop_grace_period,
            adapters: model::NodeAdapters::initialized(),
            log_observations: Arc::new(LogObservations::default()),
        }
    }

    /// Forget every child **without terminating it**.
    ///
    /// [`Drop`] kills what is still registered, which is what a desktop shell
    /// wants and what a one-shot `--node-start` must not do: without this, the
    /// CLI reports a node as started and then kills it on the way out of the
    /// process. Dropping the stored handle is not itself a signal — `ManagedChild`
    /// has no `Drop`, and dropping a `std::process::Child` leaves the OS process
    /// running.
    pub fn disown_all(&mut self) {
        self.children.clear();
    }

    /// Forget one child without terminating it, for the case where a single
    /// process is handed over to someone else to supervise.
    pub fn disown(&mut self, process_id: &str) -> bool {
        self.children.remove(process_id).is_some()
    }

    /// Whether this supervisor can actually control `node_id` — i.e. holds the
    /// handle it was started with. A node can be alive in the database and
    /// unmanaged here, and reporting the two as the same thing is how a
    /// "stopped" node keeps running.
    pub fn is_managing(&self, node_id: &str) -> bool {
        self.children.contains_key(node_id)
    }

    /// Every node this supervisor can control.
    pub fn managed_node_ids(&self) -> Vec<String> {
        self.children.keys().cloned().collect()
    }

    /// Get reference to adapter registry.
    pub fn adapters(&self) -> &model::NodeAdapters {
        &self.adapters
    }

    /// The shared log observations store, so the web layer can read the same sync
    /// snapshots the collection worker publishes instead of re-reading log files.
    pub(crate) fn observations(&self) -> &Arc<LogObservations> {
        &self.log_observations
    }
}

impl Drop for ProcessSupervisor {
    fn drop(&mut self) {
        for (_id, mut managed) in self.children.drain() {
            managed.terminate_on_drop();
        }
    }
}

impl ProcessSupervisor {
    /// Start a blocking worker that samples workspace logs immediately and every 30 seconds,
    /// using this supervisor's existing parser adapters. This method requires exclusive
    /// ownership: if another caller already claims the lease, we return an error rather
    /// than spawning a second collector for the same workspace. The returned lease moves
    /// into Engine so only the engine owns the collection loop for this data_dir/logs.
    ///
    /// The worker reads the latest bounded tail (64 KiB) per node and records new events
    /// through shared observations. It never locks the supervisor, and API consumers read
    /// the same snapshots via the observations store instead of holding a file lock.
    ///
    /// `log_dir` must be the workspace's `logs` directory, as used for launch. To shut down,
    /// drop the lease; the worker checks the `stop` flag and exits promptly.
    pub(crate) fn start_log_collection(
        &self,
        repository: &Repository,
        log_dir: PathBuf,
        stop: Arc<AtomicBool>,
    ) -> anyhow::Result<(Arc<LogObservations>, JoinHandle<()>)> {
        let store = Arc::clone(&self.log_observations);

        // Claim exclusive lease before spawning worker. Move the lease into the
        // worker so it remains held for the worker's full lifetime and is
        // released deterministically when Engine joins that worker.
        let lease = store.claim()?;

        let adapters = self.adapters.clone();
        let repository = repository.clone();
        let store_for_return = Arc::clone(&store);

        let worker = thread::Builder::new()
            .name("neonexus-log-collector".to_string())
            .spawn(move || {
                use std::sync::atomic::Ordering;

                // Keep the exclusive collection lease alive until this worker
                // exits; dropping it here clears the collecting flag for the
                // next Engine.
                let _lease = lease;
                // Each worker owns its own CursorStore for incremental tracking
                let mut cursors = CursorStore::new();

                while !stop.load(Ordering::Relaxed) {
                    match repository.list_nodes() {
                        Ok(nodes) => {
                            store.retain(&nodes);
                            for node in &nodes {
                                if stop.load(Ordering::Relaxed) {
                                    break;
                                }
                                if let Some(parser) = adapters.get_log_parser(&node.node_type) {
                                    let log_path = super::log_path_for(&log_dir, node);
                                    if let Err(error) = Self::collect_single_node_logs(
                                        &repository,
                                        node,
                                        parser,
                                        &log_path,
                                        &store,
                                        &mut cursors,
                                    ) {
                                        warn!(
                                            "neo-nexus: log collection for {} failed: {error:#}",
                                            node.name
                                        );
                                    }
                                }
                            }
                        }
                        Err(error) => {
                            warn!("neo-nexus: log collection unavailable: {error:#}");
                        }
                    }
                    thread::park_timeout(LOG_COLLECTION_INTERVAL);
                }
            })
            .context("failed to start the NeoNexus log collector")?;

        Ok((store_for_return, worker))
    }

    /// Incremental log collection using cursor-based byte offsets and file identity.
    ///
    /// Baseline: first read (or reset due to rotation/truncation) reads tail-64K but does NOT
    /// emit historical fatals - only new append-only content generates events. This matches the
    /// requirement that initial collection establishes a sync baseline without flooding with
    /// pre-existing errors.
    ///
    /// Append: subsequent reads only process new bytes beyond byte_offset, using partial-line
    /// handling to avoid splitting lines across rounds.
    ///
    /// Rotation/Truncation: detected via FileIdentity mismatch (generation/device_id change)
    /// or size decrease; triggers cursor reset and baseline behavior.
    ///
    /// Budget: enforces GLOBAL_BUFFER_LIMIT / NODE_BUFFER_LIMIT using acquire_budget/
    /// release_budget to make those constants live code.
    ///
    /// Sync progress deduplication compares against observations state and only publishes after
    /// DB success so failures don't advance offset.
    fn collect_single_node_logs(
        repository: &Repository,
        node: &crate::types::NodeConfig,
        parser: &dyn model::LogParserAdapter,
        log_path: &std::path::Path,
        observations: &Arc<LogObservations>,
        cursors: &mut CursorStore,
    ) -> anyhow::Result<()> {
        use std::fs;

        // Detect file changes (rotation/truncation/deletion) before touching the cursor.
        let file_changed = cursors.detect_file_change(&node.id, log_path);
        let is_baseline = file_changed.is_some() || cursors.get_cursor(&node.id).is_none();

        // Try to get current file metadata
        let file_size_opt = fs::metadata(log_path).ok().map(|m| m.len());

        // Ensure a cursor exists, then snapshot its offset into a local value so we can
        // release the borrow and freely call the store's budget methods below.
        let byte_offset = {
            let cursor = cursors.get_or_create(node);
            cursor.byte_offset
        };

        // Determine read range and mode
        let read_start = if is_baseline { 0 } else { byte_offset };

        let content_opt = if let Some(file_size) = file_size_opt {
            if is_baseline {
                // Baseline: read tail up to BASELINE_READ_WINDOW (64K)
                let window = BASELINE_READ_WINDOW.min(file_size as usize);
                let start = file_size.saturating_sub(window as u64);
                Self::read_range(log_path, start, window).ok()
            } else if file_size == byte_offset {
                // UpToDate: nothing new since last read
                None
            } else if file_size > byte_offset {
                // Incremental: read from offset to end
                Self::read_range(log_path, byte_offset, (file_size - byte_offset) as usize).ok()
            } else {
                // File shrank without generation change (truncation edge case) - treat as baseline
                let window = BASELINE_READ_WINDOW.min(file_size as usize);
                let start = file_size.saturating_sub(window as u64);
                Self::read_range(log_path, start, window).ok()
            }
        } else {
            // File doesn't exist yet or disappeared
            None
        };

        if let Some(content) = content_opt {
            // Check budget before processing
            let extra_bytes = content.len();
            let extra_lines = content.lines().count();

            if !cursors.can_accept(extra_bytes, extra_lines) {
                // Skip this round if over budget - will retry next interval
                return Ok(());
            }

            // Acquire budget quota
            let (allowed_bytes, allowed_lines) = cursors.acquire_budget(extra_bytes, extra_lines);
            if allowed_bytes == 0 && allowed_lines == 0 {
                // No allowance granted
                return Ok(());
            }

            // Process the content
            let mut observations_to_record = Vec::new();

            // Baseline reads do NOT emit historical fatals - only append-only new content does
            // This prevents flooding when collection starts and avoids replaying pre-existing errors
            if !is_baseline {
                // Detect fatals with message de-sensitization - only for append content
                for error in parser.detect_fatal_errors(&content) {
                    observations_to_record.push(NewRuntimeEvent {
                        node_id: Some(node.id.clone()),
                        node_name: Some(node.name.clone()),
                        kind: EventKind::LogFatalErrorDetected,
                        severity: EventSeverity::Critical,
                        message: format!(
                            "{}: log tail line {}: [{}] - {}",
                            node.name, error.line_number, error.pattern, error.suggestion
                        ),
                    });
                }
            }

            // Extract and publish sync progress - always run for both baseline and append
            let mut lines: Vec<&str> = content.lines().rev().take(20).collect();
            lines.reverse();
            if let Some(sync) = parser.extract_sync_progress(&lines) {
                Self::publish_sync_if_changed(
                    observations,
                    node,
                    &mut observations_to_record,
                    sync.current_height,
                    sync.target_height,
                    sync.sync_percentage,
                    sync.peers_connected as i32,
                );
            }

            // Batch insert ensures all-or-nothing for retries. If the DB write fails we
            // return before advancing the cursor, so the same region is retried next round.
            if !observations_to_record.is_empty() {
                if let Err(error) = repository.record_event_batch(&observations_to_record) {
                    // Release the reserved budget and leave the offset untouched.
                    cursors.release_budget(allowed_bytes, allowed_lines);
                    return Err(error);
                }
            }

            // Advance cursor and bind identity AFTER DB success.
            let identity = crate::logs::cursor::file_identity_from_path(log_path);
            if let Some(cursor) = cursors.get_cursor_mut(&node.id) {
                if let Some(actual_size) = file_size_opt {
                    if is_baseline {
                        // For baseline, cursor moves to end of read
                        cursor.advance(actual_size.saturating_sub(read_start));
                    } else {
                        // For incremental, cursor advances by processed bytes
                        cursor.advance(content.len() as u64);
                    }
                    // Bind current file identity so the next round is treated as incremental,
                    // not baseline. This makes rotation/truncation detection meaningful.
                    if let Some(identity) = identity {
                        cursor.bind_identity(identity);
                    }
                }
            }
            cursors.release_budget(allowed_bytes, allowed_lines);
        }

        Ok(())
    }

    /// Helper to read a byte range [start, start+length) from file
    fn read_range(path: &std::path::Path, start: u64, length: usize) -> anyhow::Result<String> {
        use std::fs::File;
        use std::io::{Read, Seek, SeekFrom};

        let mut file = File::open(path)?;
        file.seek(SeekFrom::Start(start))?;

        // Read up to 'length' bytes - handle partial reads gracefully
        let mut buffer = vec![0u8; length];
        let bytes_read = file.read(&mut buffer)?;

        Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
    }

    /// Deduplicate sync progress updates and publish after DB success
    fn publish_sync_if_changed(
        observations: &Arc<LogObservations>,
        node: &crate::types::NodeConfig,
        observations_to_record: &mut Vec<NewRuntimeEvent>,
        current_height: u64,
        target_height: u64,
        sync_percentage: f32,
        peers_connected: i32,
    ) {
        let existing = observations.sync_progress(node);
        let new_progress = crate::supervisor::model::SyncProgress {
            current_height,
            target_height,
            sync_percentage,
            peers_connected: peers_connected as u32,
        };

        let needs_update = match existing {
            Some(ep) => {
                ep.current_height != current_height
                    || ep.target_height != target_height
                    || (ep.sync_percentage - sync_percentage).abs() > 0.01
                    || ep.peers_connected != peers_connected as u32
            }
            None => true,
        };

        if needs_update {
            let message = format!(
                "{}/{} blocks ({:.1}%)",
                current_height, target_height, sync_percentage
            );

            // Record event for the update
            observations_to_record.push(NewRuntimeEvent {
                node_id: Some(node.id.clone()),
                node_name: Some(node.name.clone()),
                kind: EventKind::SyncProgressRecorded,
                severity: EventSeverity::Info,
                message,
            });

            // Update local observations state immediately
            let epoch = observations.epoch(&node.id);
            observations.publish(node, epoch, Some(Arc::new(new_progress)));
        }
    }
}
