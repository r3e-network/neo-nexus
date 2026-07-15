use super::*;

mod action_queue;
mod backup;
mod event_journal;
mod port_matrix;
mod readiness;

#[test]
fn workspace_integrity_action_records_report_and_event() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    app.run_workspace_integrity_check();

    let Some(report) = app.workspace_integrity_report.as_ref() else {
        anyhow::bail!("integrity report should be stored after the action");
    };
    assert!(report.is_success());
    assert!(app.session
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("Workspace integrity ok")));

    let events =
        app.repository
            .list_events(RuntimeEventFilter::new(None, "workspace-integrity", 10))?;
    assert!(events.iter().any(|event| {
        event.kind == EventKind::WorkspaceIntegrityChecked && event.severity == EventSeverity::Info
    }));

    Ok(())
}

#[test]
fn support_bundle_action_writes_archive_and_audit_event() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    repository.record_event_at(
        NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::WorkspaceIntegrityChecked,
            severity: EventSeverity::Info,
            message: "integrity checked before support export".to_string(),
        },
        42,
    )?;
    let mut app = NeoNexusApp::new(repository);

    app.export_support_bundle();

    assert!(app.session
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("Support bundle exported")));

    let support_dir = app.support_bundle_dir();
    let exported_paths = std::fs::read_dir(&support_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    assert!(exported_paths
        .iter()
        .any(|path| path.extension().is_some_and(|extension| extension == "zip")));
    let Some(bundle_dir) = exported_paths.iter().find(|path| path.is_dir()) else {
        anyhow::bail!("support bundle directory should be written");
    };
    assert!(bundle_dir.join("manifest.json").is_file());
    assert!(bundle_dir.join("privacy.txt").is_file());
    assert!(bundle_dir.join("log-diagnosis.json").is_file());
    assert!(bundle_dir.join("log-diagnosis.txt").is_file());

    let events = app
        .repository
        .list_events(RuntimeEventFilter::new(None, "support-bundle", 10))?;
    assert!(events.iter().any(|event| {
        event.kind == EventKind::SupportBundleExported && event.severity == EventSeverity::Info
    }));

    Ok(())
}

#[test]
fn release_package_actions_write_package_verify_and_audit_events() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    app.package_native_release();
    let Some(package) = app.last_release_package.as_ref() else {
        anyhow::bail!("release package should be stored after packaging");
    };
    assert!(package.archive_path.is_file());
    assert!(package.manifest_path.is_file());
    assert!(package.checksum_path.is_file());

    app.verify_native_release_package();
    assert!(app.last_release_verification.is_some());
    assert!(app.session
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("Release verified")));

    let events = app
        .repository
        .list_events(RuntimeEventFilter::new(None, "release", 10))?;
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::ReleasePackaged));
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::ReleasePackageVerified));

    Ok(())
}
