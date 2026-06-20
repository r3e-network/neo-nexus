mod checker;
mod model;
mod rules;
mod scan;

#[cfg(test)]
mod tests;

pub use checker::SourceQualityChecker;
pub use model::{SourceQualityFinding, SourceQualityReport};

pub(super) const MAX_MAINTENANCE_FILE_LINES: usize = 1000;
pub(super) const MAX_RUST_SOURCE_LINES: usize = 200;
