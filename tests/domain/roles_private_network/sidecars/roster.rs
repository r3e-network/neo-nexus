use crate::*;

#[test]
fn committee_roster_validates_compressed_public_keys() {
    let keys = format!(
        "{} {}",
        committee_public_key("02", '5'),
        committee_public_key("03", '6')
    );
    let roster = CommitteeRoster::from_public_keys(&keys).unwrap().unwrap();

    assert_eq!(roster.signers.len(), 2);
    assert_eq!(roster.signers[0].label, "committee-signer-1");
    assert_eq!(
        roster.signers[1].public_key,
        committee_public_key("03", '6')
    );

    let duplicate = format!(
        "{},{}",
        committee_public_key("02", '7'),
        committee_public_key("02", '7')
    );
    assert!(CommitteeRoster::from_public_keys(
        "04aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    )
    .unwrap_err()
    .to_string()
    .contains("start"));
    assert!(CommitteeRoster::from_public_keys("02abc")
        .unwrap_err()
        .to_string()
        .contains("compressed"));
    assert!(CommitteeRoster::from_public_keys(&duplicate)
        .unwrap_err()
        .to_string()
        .contains("duplicate"));
}

#[test]
fn committee_roster_attaches_signer_references_without_secret_material() {
    let first_key = committee_public_key("02", '8');
    let second_key = committee_public_key("03", '9');
    let keys = format!("{first_key},{second_key}");
    let signer_refs = format!(
        "{first_key}|/secure/neonexus/validator-1.wallet.json|http://127.0.0.1:9021|neo-signer --wallet {{wallet}} --endpoint {{endpoint}} --key {{public_key}}\n{second_key}|C:\\neo\\validator-2.wallet.json|https://signer.example.test/validator-2"
    );

    let roster = CommitteeRoster::from_public_keys_and_references(&keys, &signer_refs)
        .unwrap()
        .unwrap();

    assert_eq!(
        roster.signers[0].wallet_path.as_deref(),
        Some(Path::new("/secure/neonexus/validator-1.wallet.json"))
    );
    assert_eq!(
        roster.signers[0].signer_endpoint.as_deref(),
        Some("http://127.0.0.1:9021")
    );
    assert_eq!(
        roster.signers[0].signer_command_template.as_deref(),
        Some("neo-signer --wallet {wallet} --endpoint {endpoint} --key {public_key}")
    );
    let expected_command = format!(
        "neo-signer --wallet /secure/neonexus/validator-1.wallet.json --endpoint http://127.0.0.1:9021 --key {first_key}"
    );
    assert_eq!(
        roster.signers[0].signer_command.as_deref(),
        Some(expected_command.as_str())
    );
    let command_plan = roster.signers[0].signer_command_plan.as_ref().unwrap();
    assert_eq!(command_plan.execution_policy, "argv-no-shell");
    assert_eq!(command_plan.binary, "neo-signer");
    assert_eq!(command_plan.arguments[0], "--wallet");
    assert_eq!(
        command_plan.arguments[1],
        "/secure/neonexus/validator-1.wallet.json"
    );
    assert_eq!(command_plan.arguments[5], first_key);
    let incomplete_summary = roster.handoff_summary(4);
    assert_eq!(incomplete_summary.status_label(), "incomplete");
    assert_eq!(incomplete_summary.signer_count, 2);
    assert_eq!(incomplete_summary.required_signer_count, 4);
    assert_eq!(incomplete_summary.missing_required_signer_count, 2);
    assert_eq!(incomplete_summary.wallet_reference_count, 2);
    assert_eq!(incomplete_summary.missing_wallet_reference_count, 0);
    assert_eq!(incomplete_summary.endpoint_reference_count, 2);
    assert_eq!(incomplete_summary.sidecar_command_count, 1);
    assert_eq!(incomplete_summary.sidecar_command_plan_count, 1);
    assert!(incomplete_summary
        .operator_summary()
        .contains("2/4 signers"));
    assert_eq!(roster.handoff_summary(2).status_label(), "sidecars planned");

    let wallet_pending = CommitteeRoster::from_public_keys(&keys)
        .unwrap()
        .unwrap()
        .handoff_summary(2);
    assert_eq!(wallet_pending.status_label(), "wallets pending");
    assert_eq!(wallet_pending.missing_wallet_reference_count, 2);
    assert_eq!(
        roster.signers[1].wallet_path.as_deref(),
        Some(Path::new("C:\\neo\\validator-2.wallet.json"))
    );
    assert_eq!(
        roster.signers[1].signer_endpoint.as_deref(),
        Some("https://signer.example.test/validator-2")
    );
    assert!(roster.signers[1].signer_command.is_none());
    assert!(roster.signers[1].signer_command_plan.is_none());

    let unknown_key = committee_public_key("02", 'a');
    assert!(CommitteeRoster::from_public_keys_and_references(
        &first_key,
        &format!("{unknown_key}|/secure/neonexus/unknown.wallet.json|")
    )
    .unwrap_err()
    .to_string()
    .contains("unknown committee public key"));
    assert!(
        CommitteeRoster::from_public_keys_and_references(
            &first_key,
            &format!("{first_key}|/secure/neonexus/validator.wallet.json|https://user:secret@signer.example.test")
        )
        .unwrap_err()
        .to_string()
        .contains("credentials")
    );
    assert!(CommitteeRoster::from_public_keys_and_references(
        &first_key,
        &format!("{first_key}|/secure/neonexus/validator.wallet.json|ftp://signer.example.test")
    )
    .unwrap_err()
    .to_string()
    .contains("http or https"));
    assert!(
        CommitteeRoster::from_public_keys_and_references(
            &first_key,
            &format!("{first_key}|/secure/neonexus/one.wallet.json|\n{first_key}|/secure/neonexus/two.wallet.json|")
        )
        .unwrap_err()
        .to_string()
        .contains("duplicate signer reference")
    );
    assert!(CommitteeRoster::from_public_keys_and_references(
        &first_key,
        &format!("{first_key}|||neo-signer --wallet {{wallet}}")
    )
    .unwrap_err()
    .to_string()
    .contains("without a wallet path"));
    assert!(CommitteeRoster::from_public_keys_and_references(
        &first_key,
        &format!(
            "{first_key}|/secure/neonexus/validator.wallet.json||neo-signer --bad {{unknown}}"
        )
    )
    .unwrap_err()
    .to_string()
    .contains("unsupported signer sidecar command placeholder"));
    assert!(CommitteeRoster::from_public_keys_and_references(
        &first_key,
        &format!(
            "{first_key}|/secure/neonexus/validator.wallet.json||neo-signer --wallet {{wallet"
        )
    )
    .unwrap_err()
    .to_string()
    .contains("unclosed placeholder"));
    assert!(CommitteeRoster::from_public_keys_and_references(
        "",
        &format!("{first_key}|/secure/neonexus/validator.wallet.json|")
    )
    .unwrap_err()
    .to_string()
    .contains("committee public keys are required"));
}
