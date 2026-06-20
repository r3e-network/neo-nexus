mod context;
mod exporter;
mod files;
mod io;
mod logs;
mod model;
mod render;

#[cfg(test)]
mod tests;

pub use exporter::WorkspaceSupportBundleExporter;
pub use model::{
    SupportBundleFile, SupportBundleLogDiagnosis, SupportBundleLogDiagnosisReport,
    SupportBundleLogFinding, SupportBundleNode, WorkspaceSupportBundleExport,
    WorkspaceSupportBundleManifest,
};

const PRIVACY_POLICY: &str = "diagnostics-only-no-private-keys-passwords-or-webhook-secrets";
const LOG_DIAGNOSIS_MAX_EXCERPT_CHARS: usize = 160;
