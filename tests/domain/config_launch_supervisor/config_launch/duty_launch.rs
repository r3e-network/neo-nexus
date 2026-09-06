//! A node has to boot with the duty the workspace says it performs.
//!
//! Applying a duty writes the config correctly. Then Start rewrote that same
//! file with a context-free render and launched a plain relay — while the
//! workbench went on showing the duty, because the duty *was* recorded. It just
//! never reached the file the node booted from, so a consensus node relayed and
//! nothing said so.
//!
//! Both entry points did it: the GUI's start/restart and the CLI's
//! `--node-start` / `--node-restart`, since all of them route through
//! `execute_node_launch`.

use crate::*;

/// A duty applied in the workspace must survive the launch that rewrites the
/// config. Asserted on the file the node would actually read.
#[test]
fn a_launch_writes_the_config_for_the_recorded_duty() {
    let repo = create_repo();
    let node_id = create_node(&repo, "consensus-node", NodeType::NeoGo);
    repo.set_node_role(&node_id, Some(NodeRole::Consensus))
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let (registry, signer_key) = exact_wallet_registry(&temp);
    repo.set_node_signer_key(&node_id, Some(&signer_key))
        .unwrap();
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let config_path = temp.path().join("neo-go.yml");
    let plan = LaunchPlanner::plan(&node, &config_path, temp.path());
    let mut supervisor = ProcessSupervisor::default();

    // The binary does not exist, so supervision fails — but the config is
    // written first, which is the step under test.
    let _ = execute_node_launch(
        &repo,
        &mut supervisor,
        NodeLaunchRequest {
            signer_registry: Some(&registry),
            node: &node,
            plan: &plan,
            log_path: temp.path().join("node.log"),
            action: LaunchAction::Start,
            managed_config: Some(ManagedConfig {
                path: &config_path,
                plugins: &[],
            }),
        },
    );

    let written = std::fs::read_to_string(&config_path).expect("the managed config was written");
    assert!(
        written.contains("Consensus:"),
        "a node recorded as Consensus launched with a relaying config:\n{written}",
    );
}

/// Restart takes the same path, so it must not undo what Start got right.
#[test]
fn a_restart_also_writes_the_config_for_the_recorded_duty() {
    let repo = create_repo();
    let node_id = create_node(&repo, "oracle-node", NodeType::NeoGo);
    repo.set_node_role(&node_id, Some(NodeRole::Oracle))
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let (registry, signer_key) = exact_wallet_registry(&temp);
    repo.set_node_signer_key(&node_id, Some(&signer_key))
        .unwrap();
    // Restart has a real lifecycle precondition: the node must be Running and
    // reachable by a handle or recorded pid. Use a deliberately absent pid so
    // quiescing is safe and deterministic; the missing binary still exercises
    // the post-export launch failure this test wants.
    repo.update_node_status(&node_id, NodeStatus::Running, Some(4_000_000))
        .unwrap();
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let config_path = temp.path().join("neo-go.yml");
    let plan = LaunchPlanner::plan(&node, &config_path, temp.path());
    let mut supervisor = ProcessSupervisor::default();

    let _ = execute_node_launch(
        &repo,
        &mut supervisor,
        NodeLaunchRequest {
            signer_registry: Some(&registry),
            node: &node,
            plan: &plan,
            log_path: temp.path().join("node.log"),
            action: LaunchAction::Restart,
            managed_config: Some(ManagedConfig {
                path: &config_path,
                plugins: &[],
            }),
        },
    );

    let written = std::fs::read_to_string(&config_path).expect("the managed config was written");
    assert!(written.contains("Oracle:"), "{written}");
}

/// A node with no duty still gets a plain relaying config — the fix must not
/// invent a duty for nodes that have none.
#[test]
fn a_node_without_a_duty_still_launches_as_a_relay() {
    let repo = create_repo();
    let node_id = create_node(&repo, "plain-node", NodeType::NeoGo);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("neo-go.yml");
    let plan = LaunchPlanner::plan(&node, &config_path, temp.path());
    let mut supervisor = ProcessSupervisor::default();

    let _ = execute_node_launch(
        &repo,
        &mut supervisor,
        NodeLaunchRequest {
            signer_registry: None,
            node: &node,
            plan: &plan,
            log_path: temp.path().join("node.log"),
            action: LaunchAction::Start,
            managed_config: Some(ManagedConfig {
                path: &config_path,
                plugins: &[],
            }),
        },
    );

    let written = std::fs::read_to_string(&config_path).expect("written");
    for signing in ["Consensus:", "Oracle:", "StateRoot:", "P2PNotary:"] {
        assert!(
            !written.contains(signing),
            "a node with no duty got a {signing} section:\n{written}",
        );
    }
}

fn exact_wallet_registry(temp: &tempfile::TempDir) -> (SignerRegistry, SignerKeyRef) {
    const ADDRESS: &str = "NZAvnENCGsGZAN2GssPNftitKaKPfFwd5v";
    const PASSWORD: &str = "correct horse battery staple";
    const ENCRYPTED_KEY: &str = "6PYRs1PSxgTKGgoYnCfpMkb93WDTjmPgsneJgRWDxQL8D1RWjS6mAoMUxd";
    const VERIFICATION_SCRIPT: &str =
        "0c21031e18532fd4754c02f3041d9c75ceb33b83ffd81ac7ce4fe882ccb1c98bc5896e4156e7b327";
    const TESTNET_MAGIC: u32 = 894_710_606;

    let wallet_path = temp.path().join("wallet.json");
    let password_path = temp.path().join("wallet.password");
    let wallet = serde_json::json!({
        "name": "node launch fixture",
        "version": "3.0",
        "scrypt": { "n": 16_384, "r": 8, "p": 8 },
        "accounts": [{
            "address": ADDRESS,
            "label": "signer",
            "isDefault": true,
            "lock": false,
            "key": ENCRYPTED_KEY,
            "contract": {
                "script": VERIFICATION_SCRIPT,
                "parameters": [{ "name": "signature", "type": "Signature" }],
                "deployed": false
            },
            "extra": null
        }],
        "extra": null
    });
    std::fs::write(&wallet_path, serde_json::to_vec_pretty(&wallet).unwrap()).unwrap();
    std::fs::write(&password_path, format!("{PASSWORD}\n")).unwrap();
    protect_test_secret(&password_path);

    let signer = LocalWalletSigner::open(LocalWalletConfig {
        wallet_path,
        password_file: password_path,
        account: Some(ADDRESS.to_string()),
        network: "testnet".to_string(),
        network_magic: TESTNET_MAGIC,
        allow_transaction: true,
        // NeoNexus never signs consensus payloads directly with this key. The
        // native node receives the exact NEP-6 wallet and applies its own
        // role-specific protocol; the direct API remains disabled.
        allow_consensus: false,
        allow_raw: false,
    })
    .expect("fixture wallet opens");
    let key = SignerKeyRef::new("wallet", signer.key_info().key_id).unwrap();
    let profile =
        SignerBackendProfile::new("wallet", "Test wallet", SignerBackendKind::LocalWallet).unwrap();
    let backend = ConfiguredSignerBackend::local_wallet(profile, signer).unwrap();
    let registry = SignerRegistry::new([backend], None, None).unwrap();
    (registry, key)
}

fn protect_test_secret(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    #[cfg(windows)]
    {
        let account = std::env::var("USERNAME").expect("Windows test account");
        let grant = format!("{account}:(F)");
        let status = std::process::Command::new("icacls.exe")
            .arg(path)
            .args(["/inheritance:r", "/grant:r", grant.as_str()])
            .status()
            .expect("icacls runs");
        assert!(status.success(), "icacls protects the fixture secret");
    }
}
