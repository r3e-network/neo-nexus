mod diagnosis;
mod model;
mod reader;

#[cfg(test)]
#[path = "../tests/unit/logs/tests.rs"]
mod tests;

pub use self::{
    model::{LogDiagnosis, LogDiagnosisStatus, LogFinding, LogLine, LogSnapshot},
    reader::LogReader,
};
