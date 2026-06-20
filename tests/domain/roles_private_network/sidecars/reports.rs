use crate::*;

#[test]
fn launch_pack_sidecar_text_report_redacts_sensitive_display_command() {
    let report = PrivateNetworkLaunchPackSidecarReport {
        root_path: PathBuf::from("/tmp/private-pack"),
        manifest_path: PathBuf::from("/tmp/private-pack/manifest.json"),
        sidecar_count: 1,
        sidecars: vec![CommitteeSidecarProcess {
            signer_label: "committee-signer-1".to_string(),
            public_key: committee_public_key("02", 'e'),
            wallet_path: None,
            signer_endpoint: None,
            log_path: PathBuf::from("/tmp/private-pack/signers/committee-signer-1.log"),
            process: ManagedProcessSpec {
                id: "signer:committee-signer-1".to_string(),
                kind: ManagedProcessKind::Sidecar,
                label: "committee-signer-1".to_string(),
                binary_path: PathBuf::from("neo-signer"),
                args: vec![
                    "--api-key".to_string(),
                    "raw-api-key".to_string(),
                    "--wallet-password=raw-password".to_string(),
                ],
                working_dir: PathBuf::from("/tmp/private-pack"),
                display_command: "neo-signer --api-key raw-api-key --wallet-password=raw-password"
                    .to_string(),
            },
        }],
    };

    let text = report.to_cli_text();

    assert!(text.contains("command: neo-signer --api-key <redacted>"));
    assert!(text.contains("--wallet-password=<redacted>"));
    assert!(!text.contains("raw-api-key"));
    assert!(!text.contains("raw-password"));
}
