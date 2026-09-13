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

/// Every resolution has to lead somewhere, and somewhere that matches what its
/// button says. The OpsCenter labelled each finding "Open Plugins", "Open
/// Runtimes" and so on, then sent all of them to the instance page.
#[test]
fn every_resolution_leads_to_the_surface_its_label_names() {
    let encode = |value: &str| value.replace(' ', "%20");
    for resolution in DiagnosticResolution::ALL {
        let href = resolution.href("node-7", encode);
        assert!(
            href.starts_with('/'),
            "{resolution:?} produced a href that is not a path: {href}"
        );
        assert_ne!(
            href, "/nodes/node-7",
            "{resolution:?} still falls back to the instance page"
        );
    }

    // The instance-scoped surfaces carry the instance.
    for resolution in [
        DiagnosticResolution::PluginManager,
        DiagnosticResolution::RolePlanner,
        DiagnosticResolution::Logs,
        DiagnosticResolution::NodeStudio,
    ] {
        assert!(
            resolution.href("node-7", encode).contains("node-7"),
            "{resolution:?} dropped the instance it was raised for"
        );
    }

    // Names that need encoding are encoded rather than pasted into the URL.
    assert!(DiagnosticResolution::PluginManager
        .href("node 7", encode)
        .contains("node%207"));
}
