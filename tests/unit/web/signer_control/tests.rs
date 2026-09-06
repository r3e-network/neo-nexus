//! The form parsers behind the custody controls.
//!
//! These are the rules that decide whether an operator's boundary is *sent* as
//! written, and the interesting cases are all "the form said less than the
//! operator thinks": a switch that never arrived, a window with a ceiling and no
//! length, a grant list with nothing in it.
//!
//! What is deliberately not tested here is what this module used to check. Whether
//! a script hash is a script hash, what an amount may look like, which networks
//! exist — the service decides all three and says so in its own words, which are
//! then the words in the flash line. A local copy of any of those rules would be a
//! second authority to disagree with (§7 step 2), so each test below that *would*
//! have asserted a rejection now asserts the opposite: that the text is forwarded
//! untouched.

use super::*;

fn policy_form(overrides: &[(&str, &str)]) -> PolicyForm {
    let mut form = PolicyForm {
        allow_consensus: "disabled".to_string(),
        allow_raw: "disabled".to_string(),
        allow_transfer: "disabled".to_string(),
        allow_contract_call: "disabled".to_string(),
        allow_global_scope: "disabled".to_string(),
        ..PolicyForm::default()
    };
    for (name, value) in overrides {
        let slot = match *name {
            "allow_consensus" => &mut form.allow_consensus,
            "allow_raw" => &mut form.allow_raw,
            "allow_transfer" => &mut form.allow_transfer,
            "allow_contract_call" => &mut form.allow_contract_call,
            "allow_global_scope" => &mut form.allow_global_scope,
            "contract_whitelist" => &mut form.contract_whitelist,
            "contract_blacklist" => &mut form.contract_blacklist,
            "contract_method_whitelist" => &mut form.contract_method_whitelist,
            "contract_method_blacklist" => &mut form.contract_method_blacklist,
            "asset_whitelist" => &mut form.asset_whitelist,
            "asset_blacklist" => &mut form.asset_blacklist,
            "asset_limits" => &mut form.asset_limits,
            "transfer_to_whitelist" => &mut form.transfer_to_whitelist,
            "transfer_to_blacklist" => &mut form.transfer_to_blacklist,
            "max_single_amount" => &mut form.max_single_amount,
            "window_seconds" => &mut form.window_seconds,
            "window_max_amount" => &mut form.window_max_amount,
            "max_signers" => &mut form.max_signers,
            "max_system_fee" => &mut form.max_system_fee,
            "max_network_fee" => &mut form.max_network_fee,
            "signature_window_seconds" => &mut form.signature_window_seconds,
            "signature_window_count" => &mut form.signature_window_count,
            "chain_family" => &mut form.chain_family,
            "evm_max_gas_price" => &mut form.evm_max_gas_price,
            "evm_max_gas_limit" => &mut form.evm_max_gas_limit,
            "evm_method_whitelist" => &mut form.evm_method_whitelist,
            "evm_method_blacklist" => &mut form.evm_method_blacklist,
            "evm_chain_id" => &mut form.evm_chain_id,
            "additional_fields" => &mut form.additional_fields,
            other => unreachable!("no policy field named {other}"),
        };
        *slot = value.to_string();
    }
    form
}

const ASSET_A: &str = "0xef4073a0f2bacd0bc1d5de799e3b661d633eb9f9";
const ASSET_B: &str = "0x668e0c1f9d7b70a0a1c0d3e0b3f6a5d4c2e1f0a9";

#[test]
fn a_generate_form_carries_the_chain_binding_without_guessing_it() {
    let request = generate_key_request(&NewKeyForm {
        label: " NeoX treasury ".to_string(),
        network: " mainnet ".to_string(),
        network_magic: String::new(),
        chain_family: "neox".to_string(),
        chain_id: "47763".to_string(),
    })
    .expect("the chain-bound request is complete");
    assert_eq!(request.label, "NeoX treasury");
    assert_eq!(request.network, "mainnet");
    assert_eq!(request.chain_family.as_deref(), Some("neox"));
    assert_eq!(request.chain_id, Some(47_763));
    assert_eq!(request.network_magic, None);

    let legacy = generate_key_request(&NewKeyForm {
        label: "old client".to_string(),
        network: "testnet".to_string(),
        network_magic: String::new(),
        chain_family: String::new(),
        chain_id: String::new(),
    })
    .expect("blank additive fields retain the legacy request");
    assert_eq!(legacy.chain_family, None);
    assert_eq!(legacy.chain_id, None);
    assert!(generate_key_request(&NewKeyForm {
        label: "bad chain".to_string(),
        network: "testnet".to_string(),
        network_magic: String::new(),
        chain_family: "neox".to_string(),
        chain_id: "not-a-number".to_string(),
    })
    .is_err());
}

#[test]
fn a_switch_that_never_arrived_is_an_error_not_a_default() {
    // The direction this protects: a dropped field must not quietly open or
    // close a boundary, because either reading is a policy the operator never
    // wrote. The form comes back with a reason instead.
    for raw in ["", "  ", "on", "1", "yes"] {
        let error = switch(raw, "transfers")
            .err()
            .unwrap_or_else(|| unreachable!("{raw:?} is not a switch value"));
        assert!(error.to_string().contains("transfers"), "{error}");
    }
    assert_eq!(switch("enabled", "transfers").ok(), Some(true));
    assert_eq!(switch("DISABLED", "transfers").ok(), Some(false));
}

#[test]
fn a_kill_switch_reads_only_true_and_false() {
    // The page posts hidden `true`/`false` fields rather than checkbox
    // presence, so an ambiguous value is a tampered or stale form.
    assert_eq!(flag("true", "signing state").ok(), Some(true));
    assert_eq!(flag("false", "signing state").ok(), Some(false));
    // Surrounding whitespace is machine noise and is trimmed; nothing else is.
    assert_eq!(flag(" true ", "signing state").ok(), Some(true));
    for raw in ["", "True", "TRUE", "on", "1", "enabled"] {
        assert!(flag(raw, "signing state").is_err(), "accepted {raw:?}");
    }
}

#[test]
fn a_blank_boundary_is_sent_as_everything_closed() {
    // §4.2's default-closed reading is the service's, and the form's job is to
    // ask for it: every switch named and nothing extra in the lists.
    let policy = build_policy(&policy_form(&[]))
        .ok()
        .unwrap_or_else(|| unreachable!("an all-disabled form with empty lists is complete"));
    assert!(!policy.allow_transfer);
    assert!(!policy.allow_consensus);
    assert!(!policy.allow_raw);
    assert!(!policy.allow_contract_call);
    assert!(!policy.allow_global_scope);
    assert!(policy.contract_whitelist.is_empty());
    assert_eq!(policy.max_single_amount, None);
    assert_eq!(policy.window_limit, None);
}

#[test]
fn a_window_needs_both_ends_of_it() {
    // One half of a rolling limit is a limit that does nothing, and the
    // operator would never learn which transfers it had failed to constrain.
    let only_ceiling = policy_form(&[("allow_transfer", "enabled"), ("window_max_amount", "500")]);
    let error = build_policy(&only_ceiling)
        .err()
        .unwrap_or_else(|| unreachable!("a ceiling with no window length means nothing"));
    assert!(error.to_string().contains("length in seconds"), "{error}");

    let only_seconds = policy_form(&[("allow_transfer", "enabled"), ("window_seconds", "3600")]);
    let error = build_policy(&only_seconds)
        .err()
        .unwrap_or_else(|| unreachable!("a window with no ceiling means nothing"));
    assert!(error.to_string().contains("ceiling"), "{error}");

    let both = policy_form(&[
        ("allow_transfer", "enabled"),
        ("window_seconds", "3600"),
        ("window_max_amount", "500"),
    ]);
    let policy = build_policy(&both)
        .ok()
        .unwrap_or_else(|| unreachable!("complete"));
    assert_eq!(
        policy.window_limit,
        Some(WindowLimit {
            seconds: 3600,
            max_amount: "500".to_string(),
        })
    );
}

#[test]
fn every_current_policy_field_and_future_extension_survives_the_form() {
    let methods = format!(r#"[{{"contract":"{ASSET_A}","method":"transfer"}}]"#);
    let asset_limits = format!(
        r#"[{{"asset":"{ASSET_A}","max_single_amount":"10","window_limit":{{"seconds":60,"max_amount":"100"}}}}]"#
    );
    let form = policy_form(&[
        ("allow_transfer", "enabled"),
        ("contract_method_whitelist", &methods),
        ("asset_limits", &asset_limits),
        ("max_signers", "2"),
        ("max_system_fee", "100000000"),
        ("max_network_fee", "20000000"),
        ("signature_window_seconds", "60"),
        ("signature_window_count", "5"),
        ("chain_family", "neox"),
        ("evm_max_gas_price", "30000000000"),
        ("evm_max_gas_limit", "250000"),
        ("evm_method_whitelist", "0xa9059cbb"),
        ("evm_method_blacklist", "0x095ea7b3"),
        ("evm_chain_id", "47763"),
        (
            "additional_fields",
            r#"{"future_daily_request_limit":{"seconds":86400,"count":7}}"#,
        ),
    ]);

    let policy = build_policy(&form).expect("the full policy form is valid");
    assert_eq!(policy.contract_method_whitelist[0].method, "transfer");
    assert_eq!(
        policy.asset_limits[0]
            .window_limit
            .as_ref()
            .unwrap()
            .seconds,
        60
    );
    assert_eq!(policy.max_signers, Some(2));
    assert_eq!(policy.max_system_fee.as_deref(), Some("100000000"));
    assert_eq!(policy.max_network_fee.as_deref(), Some("20000000"));
    assert_eq!(
        policy.max_signatures,
        Some(SignatureRateLimit {
            seconds: 60,
            count: 5,
        })
    );
    assert_eq!(policy.chain_family.as_deref(), Some("neox"));
    assert_eq!(policy.evm_max_gas_price.as_deref(), Some("30000000000"));
    assert_eq!(policy.evm_max_gas_limit, Some(250_000));
    assert_eq!(policy.evm_method_whitelist, vec!["0xa9059cbb"]);
    assert_eq!(policy.evm_method_blacklist, vec!["0x095ea7b3"]);
    assert_eq!(policy.evm_chain_id, Some(47_763));
    assert_eq!(
        policy.additional_fields["future_daily_request_limit"]["count"],
        7
    );
}

#[test]
fn nested_policy_json_and_signature_windows_fail_closed() {
    let malformed = policy_form(&[("asset_limits", "not-json")]);
    let error = build_policy(&malformed).expect_err("invalid nested policy cannot be dropped");
    assert!(error.to_string().contains("asset limits"), "{error:#}");

    let incomplete = policy_form(&[("signature_window_seconds", "60")]);
    let error = build_policy(&incomplete).expect_err("half a rate limit is not a limit");
    assert!(error.to_string().contains("needs a count"), "{error:#}");

    let malformed_extensions = policy_form(&[("additional_fields", "[]")]);
    let error = build_policy(&malformed_extensions)
        .expect_err("extensions must be an object so their field names survive");
    assert!(error.to_string().contains("JSON object"), "{error:#}");
}

#[test]
fn a_pasted_list_survives_every_separator_an_operator_uses() {
    let form = policy_form(&[
        ("allow_transfer", "enabled"),
        (
            "asset_whitelist",
            &format!("{ASSET_A}, {ASSET_B}\r\n{ASSET_A}"),
        ),
    ]);
    let policy = build_policy(&form)
        .ok()
        .unwrap_or_else(|| unreachable!("three valid hashes over two lines"));
    assert_eq!(
        policy.asset_whitelist,
        vec![
            ASSET_A.to_string(),
            ASSET_B.to_string(),
            ASSET_A.to_string()
        ],
        "the list arrives as it was pasted; sorting, deduplicating and rejecting \
         duplicates is the service's normalizing to do"
    );
}

#[test]
fn an_entry_that_is_not_a_hash_is_forwarded_rather_than_falled_over_locally() {
    // This is the case the old code refused here. It is now the service's
    // `signer-request-unreadable` naming its own field, and the console's only
    // contribution is that the bad entry reaches the request rather than being
    // dropped — a list field that quietly lost an entry would save a boundary
    // narrower than the one on screen.
    let form = policy_form(&[("asset_whitelist", &format!("{ASSET_A}, typo"))]);
    let policy = build_policy(&form)
        .ok()
        .unwrap_or_else(|| unreachable!("the form is complete; the hash is the service's problem"));
    assert_eq!(policy.asset_whitelist.len(), 2);
    assert_eq!(policy.asset_whitelist[1], "typo");
}

#[test]
fn an_amount_is_carried_as_the_text_typed_with_blank_read_as_no_ceiling() {
    assert_eq!(amount(""), None);
    assert_eq!(amount("  "), None);
    assert_eq!(amount(" 1000 "), Some("1000"));
    // All three of these are the service's refusals to make, and the largest
    // value in the contract is one the console must not touch: an `i128` written
    // as a JSON number loses its high digits somewhere in a client's parser,
    // which is why §5.1 carries amounts as text at all.
    assert_eq!(amount("1.5"), Some("1.5"));
    assert_eq!(amount("1e6"), Some("1e6"));
    assert_eq!(
        amount("170141183460469231731687303715884105727"),
        Some("170141183460469231731687303715884105727")
    );

    let policy = build_policy(&policy_form(&[("max_single_amount", "-5")]))
        .ok()
        .unwrap_or_else(|| unreachable!("a negative ceiling is stored and then advised about"));
    // Blank and absent are the same request; `Some("")` would be a field the
    // service has to fail to parse rather than a ceiling nobody wrote.
    assert_eq!(policy.max_single_amount, Some("-5".to_string()));
    assert_eq!(
        build_policy(&policy_form(&[]))
            .ok()
            .unwrap_or_else(|| unreachable!("complete"))
            .max_single_amount,
        None
    );
}

#[test]
fn a_grant_names_keys_rather_than_labels() {
    assert!(grant("any", &[]).ok().is_some_and(|g| g.is_any()));
    assert_eq!(
        grant("only", &["key-1".to_string(), " key-2 ".to_string()]).ok(),
        Some(Grant::only(["key-1", "key-2"]))
    );
    // Nothing selected grants nothing. Reading an empty list as "any key" is
    // how a whitelist becomes a master key.
    assert_eq!(
        grant("only", &[]).ok(),
        Some(Grant::only(Vec::<String>::new()))
    );
    assert!(grant("", &[]).is_err());
    assert!(grant("all", &[]).is_err());
    // The whole-vault grant has no key list, whichever way the form was filled:
    // a mode of `any` with ids beside it is a record the service would have to
    // read one of.
    assert!(grant("any", &["key-1".to_string()])
        .ok()
        .is_some_and(|grant| grant.key_ids.is_empty()));
}

#[test]
fn a_workload_form_keeps_every_selected_key_and_rejects_ambiguous_scalars() {
    let form = workload_caller_form(
        b"label=validator&grant=only&keys=key-1&keys=key-2&capability=sign&origins=https%3A%2F%2Fconsole.example&workload_public_key=d75a&workload_subject=validator-01",
    )
    .expect("a multi-key workload grant is a valid form");

    assert_eq!(form.label, "validator");
    assert_eq!(form.grant, "only");
    assert_eq!(form.keys, ["key-1", "key-2"]);
    assert_eq!(form.capability, "sign");
    assert_eq!(form.origins, "https://console.example");
    assert_eq!(form.workload_public_key, "d75a");
    assert_eq!(form.workload_subject, "validator-01");

    let error = workload_caller_form(b"label=first&label=second")
        .err()
        .expect("two scalar values must never be collapsed into one identity");
    assert!(
        error.to_string().contains("duplicate scalar field"),
        "{error}"
    );
}
