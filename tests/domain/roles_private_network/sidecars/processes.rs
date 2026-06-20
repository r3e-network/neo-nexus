use crate::*;

#[test]
fn committee_roster_builds_native_sidecar_process_specs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let first_key = committee_public_key("02", 'c');
    let second_key = committee_public_key("03", 'd');
    let keys = format!("{first_key},{second_key}");
    let signer_refs = format!(
        "{first_key}|wallets/validator-1.wallet.json|http://127.0.0.1:9021|signer-bin/neo-signer --wallet {{wallet}} --listen {{endpoint}} --label {{label}}\n{second_key}|wallets/validator-2.wallet.json||neo-signer --wallet {{wallet}} --key {{public_key}}"
    );
    let launch_pack_root = temp_dir.path().join("private-pack");
    let roster = CommitteeRoster::from_public_keys_and_references(&keys, &signer_refs)
        .unwrap()
        .unwrap();

    let sidecars = roster.sidecar_processes(&launch_pack_root).unwrap();

    assert_eq!(sidecars.len(), 2);
    assert_eq!(sidecars[0].signer_label, "committee-signer-1");
    assert_eq!(sidecars[0].public_key, first_key);
    assert_eq!(
        sidecars[0].wallet_path.as_deref(),
        Some(Path::new("wallets/validator-1.wallet.json"))
    );
    assert_eq!(
        sidecars[0].signer_endpoint.as_deref(),
        Some("http://127.0.0.1:9021")
    );
    assert_eq!(sidecars[0].process.id, "signer:committee-signer-1");
    assert_eq!(sidecars[0].process.kind, ManagedProcessKind::Sidecar);
    assert_eq!(
        sidecars[0].process.binary_path,
        launch_pack_root.join("signer-bin").join("neo-signer")
    );
    assert_eq!(sidecars[0].process.working_dir, launch_pack_root);
    assert_eq!(sidecars[0].process.args[0], "--wallet");
    assert_eq!(
        sidecars[0].process.args[1],
        "wallets/validator-1.wallet.json"
    );
    assert_eq!(
        sidecars[0].log_path,
        temp_dir
            .path()
            .join("private-pack")
            .join("signers")
            .join("committee-signer-1")
            .join("committee-signer-1.supervisor.log")
    );
    assert!(sidecars[0]
        .process
        .display_command
        .contains("signer-bin/neo-signer"));

    assert_eq!(sidecars[1].process.id, "signer:committee-signer-2");
    assert_eq!(sidecars[1].process.binary_path, PathBuf::from("neo-signer"));
    assert_eq!(
        sidecars[1].process.args[1],
        "wallets/validator-2.wallet.json"
    );
    assert_eq!(sidecars[1].process.args[3], second_key);
    assert!(sidecars[1].signer_endpoint.is_none());
}
