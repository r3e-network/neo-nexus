use super::*;

#[test]
fn diagnostic_check_key_includes_resolution_identity() {
    let runtime_check = DiagnosticCheck::new(
        CheckSeverity::Critical,
        "Binary",
        "same detail",
        DiagnosticResolution::RuntimeManager,
    );
    let plugin_check = DiagnosticCheck::new(
        CheckSeverity::Critical,
        "Binary",
        "same detail",
        DiagnosticResolution::PluginManager,
    );

    let key = runtime_check.key();

    assert!(key.matches(&runtime_check));
    assert!(!key.matches(&plugin_check));
}
