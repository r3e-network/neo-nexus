mod cleanup;
pub(crate) mod cursor;
pub(crate) mod diagnosis;
pub(crate) mod model;
pub(crate) mod observations;
pub(crate) mod reader;

#[cfg(test)]
#[path = "../tests/unit/logs/tests.rs"]
mod tests;

pub use self::cleanup::clear_all_logs;
pub use self::{
    model::{LogDiagnosis, LogDiagnosisStatus, LogFinding, LogLine, LogSnapshot},
    reader::LogReader,
};
