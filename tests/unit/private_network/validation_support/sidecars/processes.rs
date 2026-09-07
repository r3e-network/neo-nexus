//! Tests for committee sidecar process specification generation

use std::path::Path;

use crate::private_network::{committee_sidecar_process, SignerCommandPlan};

fn valid_plan() -> SignerCommandPlan {
    SignerCommandPlan {
        execution_policy: "argv-no-shell".to_string(),
        binary: "neo-signer".to_string(),
        arguments: vec!["--port".to_string(), "20333".to_string()],
    }
}

#[test]
fn committee_sidecar_process_builds_spec_from_plan() {
    let root = Path::new("/srv/launch-pack");
    let sidecar = committee_sidecar_process(
        root,
        "committee-signer-1",
        "03abc",
        None,
        None,
        &valid_plan(),
    )
    .expect("valid plan produces a sidecar spec");

    assert_eq!(sidecar.signer_label, "committee-signer-1");
    assert_eq!(sidecar.public_key, "03abc");
    assert_eq!(sidecar.process.id, "signer:committee-signer-1");
    assert_eq!(sidecar.process.args, vec!["--port", "20333"]);
    assert!(
        sidecar
            .log_path
            .ends_with("committee-signer-1.supervisor.log"),
        "log path should be labelled per signer: {}",
        sidecar.log_path.display()
    );
}

#[test]
fn committee_sidecar_process_carries_wallet_and_endpoint_metadata() {
    let root = Path::new("/srv/launch-pack");
    let sidecar = committee_sidecar_process(
        root,
        "committee-signer-2",
        "02def",
        Some("wallets/c2.json".into()),
        Some("http://127.0.0.1:9000".to_string()),
        &valid_plan(),
    )
    .expect("valid plan produces a sidecar spec");

    assert_eq!(
        sidecar.wallet_path.as_deref(),
        Some(Path::new("wallets/c2.json"))
    );
    assert_eq!(
        sidecar.signer_endpoint.as_deref(),
        Some("http://127.0.0.1:9000")
    );
}

#[test]
fn committee_sidecar_process_rejects_invalid_execution_policy() {
    let root = Path::new("/srv/launch-pack");
    let mut plan = valid_plan();
    plan.execution_policy = "shell".to_string();
    let result = committee_sidecar_process(root, "signer", "03abc", None, None, &plan);
    assert!(
        result.is_err(),
        "non-argv execution policy must be rejected"
    );
}
