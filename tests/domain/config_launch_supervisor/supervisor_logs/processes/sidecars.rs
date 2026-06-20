use super::*;

#[cfg(unix)]
#[test]
fn supervisor_starts_generic_sidecar_process_with_stable_logs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let sidecar_log_path = temp_dir.path().join("logs").join("committee-signer.log");
    let sidecar_spec = ManagedProcessSpec {
        id: "signer:committee-1".to_string(),
        kind: ManagedProcessKind::Sidecar,
        label: "committee signer 1".to_string(),
        binary_path: PathBuf::from("/bin/sh"),
        args: vec![
            "-c".to_string(),
            "echo signer-ready; echo signer-warning >&2; exit 3".to_string(),
        ],
        working_dir: temp_dir.path().join("signers").join("committee-1"),
        display_command: "/bin/sh -c 'echo signer-ready; echo signer-warning >&2; exit 3'"
            .to_string(),
    };
    let mut supervisor = ProcessSupervisor::default();

    let start = supervisor
        .start_process(&sidecar_spec, &sidecar_log_path)
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    let exits = supervisor.reap_finished().unwrap();

    let text = std::fs::read_to_string(&sidecar_log_path).unwrap();
    assert_eq!(start.log_path, sidecar_log_path);
    assert!(sidecar_spec.working_dir.exists());
    assert_eq!(exits.len(), 1);
    assert_eq!(exits[0].process_id, sidecar_spec.id);
    assert_eq!(exits[0].pid, start.pid);
    assert_eq!(exits[0].exit_code, Some(3));
    assert!(text.contains("process-id: signer:committee-1"));
    assert!(text.contains("process-kind: sidecar"));
    assert!(text.contains("sidecar: committee signer 1"));
    assert!(text.contains("signer-ready"));
    assert!(text.contains("signer-warning"));
}

#[cfg(unix)]
#[test]
fn supervisor_redacts_sensitive_sidecar_display_command_in_logs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let sidecar_log_path = temp_dir.path().join("logs").join("committee-signer.log");
    let sidecar_spec = ManagedProcessSpec {
        id: "signer:committee-secret".to_string(),
        kind: ManagedProcessKind::Sidecar,
        label: "committee secret signer".to_string(),
        binary_path: PathBuf::from("/bin/sh"),
        args: vec!["-c".to_string(), "echo signer-ready".to_string()],
        working_dir: temp_dir.path().join("signers").join("committee-secret"),
        display_command:
            "/bin/sh --api-key raw-api-key --wallet-password=raw-password --seed raw-seed"
                .to_string(),
    };
    let mut supervisor = ProcessSupervisor::default();

    supervisor
        .start_process(&sidecar_spec, &sidecar_log_path)
        .unwrap();
    thread::sleep(Duration::from_millis(100));
    let _ = supervisor.reap_finished().unwrap();

    let text = std::fs::read_to_string(&sidecar_log_path).unwrap();
    assert!(text.contains("command: /bin/sh --api-key <redacted>"));
    assert!(text.contains("--wallet-password=<redacted>"));
    assert!(text.contains("--seed <redacted>"));
    assert!(!text.contains("raw-api-key"));
    assert!(!text.contains("raw-password"));
    assert!(!text.contains("raw-seed"));
}
