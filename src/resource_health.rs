//! Background disk-capacity and available-memory observations, never an
//! automatic delete/stop policy. Storage I/O runs outside the guardian thread.
mod collector;
mod model;
mod monitor;
mod render;

pub use model::{ResourcePolicy, ResourceReading, ResourceReport, ResourceStatus};
pub(crate) use monitor::settle;
pub use monitor::ResourceMonitor;
pub use render::prometheus;

#[cfg(test)]
#[path = "../tests/unit/resource_health.rs"]
mod tests;
