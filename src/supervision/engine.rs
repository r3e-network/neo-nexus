//! The engine's thread and its shutdown contract.
//!
//! Dropping the handle stops the loop and waits for the thread, so a
//! shutting-down server cannot leave a probe mid-flight.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Context;

use super::{
    observation::run_observation_loop,
    startup::reconcile_startup,
    state::{EngineState, LoopState},
};

/// How often the loop wakes. Every interval it enforces is a multiple of this
/// or is compared against `Instant`, so a second keeps latency invisible while
/// leaving the tick cheap.
const TICK: Duration = Duration::from_secs(1);

/// Handle to the running engine.
pub struct Engine {
    pub(super) stop: Arc<AtomicBool>,
    pub(super) worker: Option<JoinHandle<()>>,
    pub(super) log_collection_stop: Arc<AtomicBool>,
    pub(super) log_collection_handle: Option<JoinHandle<()>>,
    pub(super) observation_stop: Arc<AtomicBool>,
    pub(super) observation_handle: Option<JoinHandle<()>>,
}

impl Engine {
    pub fn start(state: EngineState) -> anyhow::Result<Self> {
        // Before the first page can be served: a workspace reopened after a
        // crash still claims nodes are Running, and the operator should never
        // see a status the host does not back.
        reconcile_startup(&state);

        // The incremental log collector is owned and joined by this Engine, so a
        // shutting-down server never leaves a collector sampling files. It shares
        // the one supervisor's adapters and observations; the wake-able stop flag
        // lets `Drop` cut short the 30-second sampling wait.
        let log_collection_stop = Arc::new(AtomicBool::new(false));
        let log_dir = state.workspace_child_dir("logs");
        let log_worker = {
            let supervisor = state.supervisor();
            let (_observations, worker) = supervisor.start_log_collection(
                &state.repository,
                log_dir,
                Arc::clone(&log_collection_stop),
            )?;
            worker
        };

        // Chain observation gets its own thread because a pass is blocking I/O
        // bounded by nodes-per-pass × timeout. On the supervision thread that
        // would put a dozen seconds of unreachable-node timeouts in front of
        // every crash restart, which is the one thing the tick must do promptly.
        let observation_stop = Arc::new(AtomicBool::new(false));
        let observation_handle = {
            let state = state.clone();
            let stop = Arc::clone(&observation_stop);
            thread::Builder::new()
                .name("neonexus-observation".to_string())
                .spawn(move || run_observation_loop(state, stop))
                .context("failed to start the NeoNexus observation loop")?
        };

        let stop = Arc::new(AtomicBool::new(false));
        let closing = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("neonexus-supervision".to_string())
            .spawn(move || {
                let mut loop_state = LoopState::bootstrap(&state);
                while !closing.load(Ordering::Relaxed) {
                    loop_state.tick(&state);
                    thread::sleep(TICK);
                }
            })
            .context("failed to start the NeoNexus supervision engine")?;

        Ok(Self {
            stop,
            worker: Some(worker),
            log_collection_stop,
            log_collection_handle: Some(log_worker),
            observation_stop,
            observation_handle: Some(observation_handle),
        })
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // Stop every worker
        self.stop.store(true, Ordering::Relaxed);
        self.log_collection_stop.store(true, Ordering::Relaxed);
        self.observation_stop.store(true, Ordering::Relaxed);

        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        if let Some(observer) = self.observation_handle.take() {
            // A pass already in flight finishes first: its calls carry the
            // policy's timeout, so the wait is bounded, and abandoning a
            // half-written round would leave the sample and its legacy row
            // disagreeing.
            let _ = observer.join();
        }
        if let Some(log_worker) = self.log_collection_handle.take() {
            // The collector spends most of its life in a 30-second `park_timeout`;
            // unpark it so shutdown cuts the wait short instead of blocking a join.
            log_worker.thread().unpark();
            let _ = log_worker.join();
        }
    }
}
