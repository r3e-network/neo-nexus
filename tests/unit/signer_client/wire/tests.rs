use serde_json::Value;

use super::{
    AssetLimit, Caller, ContractMethod, CreatedCaller, Eip191FulfillmentSignature,
    GenerateKeyRequest, Grant, KeyBoundary, KeyPublic, Outcome, Policy, RawSignature, Refusal,
    SavedBoundary, SignRequest, Signature, SignatureRateLimit, WindowLimit,
};

#[test]
fn an_amount_survives_the_wire_as_text_not_as_a_number() {
    // 3.4e38 raw units: a JSON number cannot carry an i128, and a ceiling that
    // quietly lost its high digits would still render as a ceiling an operator
    // believed in.
    let ceiling = "170141183460469231731687303715884105727";
    let policy = Policy {
        allow_transfer: true,
        max_single_amount: Some(ceiling.to_string()),
        window_limit: Some(WindowLimit {
            seconds: 3600,
            max_amount: ceiling.to_string(),
        }),
        ..Policy::default()
    };
    let text = serde_json::to_string(&policy).expect("a boundary encodes");
    assert!(text.contains(ceiling), "{text}");
    let read: Policy = serde_json::from_str(&text).expect("the same text reads back");
    assert_eq!(read, policy);
}

#[test]
fn a_boundary_names_exactly_the_fields_the_service_will_accept() {
    // Every request body on §5.1 is `deny_unknown_fields`, so this is the drift
    // guard for the one type used both ways: a field renamed here is a
    // `signer-request-unreadable` there, and a field dropped here is a boundary
    // that quietly stopped being written.
    let represented = Policy {
        chain_family: Some("neox".to_string()),
        evm_max_gas_price: Some("1000000000".to_string()),
        evm_max_gas_limit: Some(21_000),
        evm_method_whitelist: vec!["0xa9059cbb".to_string()],
        evm_method_blacklist: vec!["0x095ea7b3".to_string()],
        evm_chain_id: Some(47_763),
        ..Policy::default()
    };
    let mut names: Vec<String> = serde_json::to_value(represented)
        .expect("a default boundary encodes")
        .as_object()
        .expect("an object")
        .keys()
        .cloned()
        .collect();
    names.sort();
    // Sorted on both sides: whether serde_json preserves insertion order depends
    // on a feature flag, and this is a statement about the set of fields.
    let mut expected = vec![
        "allow_consensus",
        "allow_raw",
        "allow_transfer",
        "allow_contract_call",
        "allow_global_scope",
        "contract_whitelist",
        "contract_blacklist",
        "contract_method_whitelist",
        "contract_method_blacklist",
        "asset_whitelist",
        "asset_blacklist",
        "asset_limits",
        "transfer_to_whitelist",
        "transfer_to_blacklist",
        "max_single_amount",
        "window_limit",
        "max_signers",
        "max_system_fee",
        "max_network_fee",
        "max_signatures",
        "chain_family",
        "evm_max_gas_price",
        "evm_max_gas_limit",
        "evm_method_whitelist",
        "evm_method_blacklist",
        "evm_chain_id",
    ];
    expected.sort();
    expected.dedup();
    assert_eq!(names, expected);
}

#[test]
fn a_boundary_that_names_nothing_is_a_boundary_that_permits_nothing() {
    // §4.2's "blank means everything closed", restated on the wire: the defaults
    // here have to be the service's defaults, or a console form submitted empty
    // would mean two different things depending on which side guessed.
    let empty: Policy = serde_json::from_str("{}").expect("an empty body is a boundary");
    assert_eq!(empty, Policy::default());
    assert!(!empty.allow_consensus);
    assert!(!empty.allow_raw);
    assert!(!empty.allow_transfer);
    assert!(!empty.allow_contract_call);
    assert!(!empty.allow_global_scope);
    assert!(empty.contract_whitelist.is_empty());
    assert!(empty.contract_method_whitelist.is_empty());
    assert!(empty.asset_limits.is_empty());
    assert!(empty.max_single_amount.is_none());
    assert!(empty.window_limit.is_none());
    assert!(empty.max_signatures.is_none());
    assert!(empty.chain_family.is_none());
    assert!(empty.evm_method_whitelist.is_empty());
    assert!(empty.additional_fields.is_empty());
}

#[test]
fn the_complete_current_policy_survives_a_round_trip() {
    let policy = Policy {
        allow_transfer: true,
        contract_method_whitelist: vec![ContractMethod {
            contract: "0xef4073a0f2bacd0bc1d5de799e3b661d633eb9f9".to_string(),
            method: "transfer".to_string(),
        }],
        asset_limits: vec![AssetLimit {
            asset: "0xef4073a0f2bacd0bc1d5de799e3b661d633eb9f9".to_string(),
            max_single_amount: Some("1000".to_string()),
            window_limit: Some(WindowLimit {
                seconds: 3600,
                max_amount: "5000".to_string(),
            }),
        }],
        max_signers: Some(2),
        max_system_fee: Some("100000000".to_string()),
        max_network_fee: Some("20000000".to_string()),
        max_signatures: Some(SignatureRateLimit {
            seconds: 60,
            count: 5,
        }),
        chain_family: Some("neox".to_string()),
        evm_max_gas_price: Some("30000000000".to_string()),
        evm_max_gas_limit: Some(250_000),
        evm_method_whitelist: vec!["0xa9059cbb".to_string()],
        evm_method_blacklist: vec!["0x095ea7b3".to_string()],
        evm_chain_id: Some(47_763),
        ..Policy::default()
    };
    let encoded = serde_json::to_string(&policy).expect("policy encodes");
    let decoded: Policy = serde_json::from_str(&encoded).expect("policy decodes");
    assert_eq!(decoded, policy);
}

#[test]
fn additive_policy_fields_are_not_lost_by_a_known_field_edit() {
    let mut policy: Policy = serde_json::from_str(
        r#"{"allow_transfer":true,"future_daily_request_limit":{"count":7,"seconds":86400}}"#,
    )
    .expect("a newer signer policy still parses");
    assert_eq!(
        policy.additional_fields["future_daily_request_limit"]["count"],
        7
    );
    policy.allow_transfer = false;
    let encoded = serde_json::to_value(policy).expect("the edited policy encodes");
    assert_eq!(encoded["future_daily_request_limit"]["seconds"], 86_400);
}

#[test]
fn signing_requests_carry_idempotency_and_chain_identity_only_when_named() {
    let legacy = serde_json::to_value(SignRequest::neo_n3("key-1", "00")).unwrap();
    assert_eq!(legacy.as_object().map(|object| object.len()), Some(2));

    let explicit = SignRequest {
        key_id: "key-x".to_string(),
        unsigned_hex: "02f8".to_string(),
        request_id: Some("req-41".to_string()),
        chain_family: Some("neox".to_string()),
        chain_id: Some(47_763),
    };
    let encoded = serde_json::to_value(explicit).unwrap();
    assert_eq!(encoded["request_id"], "req-41");
    assert_eq!(encoded["chain_family"], "neox");
    assert_eq!(encoded["chain_id"], 47_763);
}

#[test]
fn key_generation_keeps_the_legacy_body_and_can_name_a_neox_chain() {
    let legacy =
        serde_json::to_value(GenerateKeyRequest::neo_n3("relay", "testnet", None)).unwrap();
    assert_eq!(legacy.as_object().map(|object| object.len()), Some(2));

    let explicit = GenerateKeyRequest {
        label: "neo-x treasury".to_string(),
        network: "mainnet".to_string(),
        chain_family: Some("neox".to_string()),
        chain_id: Some(47_763),
        network_magic: None,
    };
    let encoded = serde_json::to_value(explicit).unwrap();
    assert_eq!(encoded["chain_family"], "neox");
    assert_eq!(encoded["chain_id"], 47_763);
    assert!(encoded.get("network_magic").is_none());
}

#[test]
fn a_chain_bound_key_keeps_identity_and_additive_metadata() {
    let key: KeyPublic = serde_json::from_str(
        r#"{"key_id":"key-x","label":"relay","network":"mainnet","network_magic":860833102,"chain_family":"neox","chain_id":47763,"public_key":"02ab","script_hash":"0x12","address":"0xab","verification_script":"","signing_enabled":true,"attestation":{"pcr0":"aa"}}"#,
    )
    .expect("a current chain-bound key parses");
    assert_eq!(key.chain_family.as_deref(), Some("neox"));
    assert_eq!(key.chain_id, Some(47_763));
    assert_eq!(key.additional_fields["attestation"]["pcr0"], "aa");

    let relayed = serde_json::to_value(key).expect("the key re-encodes");
    assert_eq!(relayed["chain_family"], "neox");
    assert_eq!(relayed["chain_id"], 47_763);
    assert_eq!(relayed["attestation"]["pcr0"], "aa");

    let legacy: KeyPublic = serde_json::from_str(
        r#"{"key_id":"key-1","label":"old","network":"testnet","public_key":"02ab","script_hash":"0x12","address":"NcgY","verification_script":"0c21","signing_enabled":true}"#,
    )
    .expect("a pre-chain-bound signer remains readable");
    assert_eq!(legacy.chain_family, None);
    assert_eq!(legacy.chain_id, None);
}

#[test]
fn a_neox_signature_keeps_its_fields_and_additive_response_data() {
    let signature: Signature = serde_json::from_str(
        r#"{"key_id":"key-x","script_hash":"0x12","address":"0xab","digest":"cafe","chain_family":"neox","chain_id":47763,"signed_transaction":"02f8…","signature":"11","public_key":"04ab","future_receipt":{"type":2}}"#,
    )
    .expect("a NeoX answer does not need Neo N3 witness fields");
    assert_eq!(signature.invocation_script, None);
    assert_eq!(signature.verification_script, None);
    assert_eq!(signature.chain_family.as_deref(), Some("neox"));
    assert_eq!(signature.signed_transaction.as_deref(), Some("02f8…"));
    assert_eq!(signature.signature.as_deref(), Some("11"));
    assert_eq!(signature.public_key.as_deref(), Some("04ab"));
    assert_eq!(signature.additional_fields["future_receipt"]["type"], 2);

    let relayed = serde_json::to_value(signature).expect("the response re-encodes");
    assert_eq!(relayed["future_receipt"]["type"], 2);
    assert!(relayed.get("invocation_script").is_none());
    assert!(relayed.get("verification_script").is_none());
}

#[test]
fn an_eip191_fulfillment_response_keeps_derived_hashes_and_future_fields() {
    let signature: Eip191FulfillmentSignature = serde_json::from_str(
        r#"{"key_id":"key-x","address":"0x1234","public_key":"02ab","digest":"0x01","message_hash":"0x02","signature":"0x03","chain_family":"neox","chain_id":47763,"oracle_contract":"0x2222222222222222222222222222222222222222","future_proof":{"version":2}}"#,
    )
    .expect("the semantic signature parses");
    assert_eq!(signature.chain_family, "neox");
    assert_eq!(signature.chain_id, 47_763);
    assert_eq!(signature.additional_fields["future_proof"]["version"], 2);
    let relayed = serde_json::to_value(signature).expect("the response re-encodes");
    assert_eq!(relayed["message_hash"], "0x02");
    assert_eq!(relayed["future_proof"]["version"], 2);
}

#[test]
fn a_raw_signature_carries_the_v1_verification_fields() {
    let raw: RawSignature = serde_json::from_str(
        r#"{"key_id":"key-1","script_hash":"0x1a2b","address":"NcgY","digest":"aabb","signature":"11","public_key":"02ab","invocation_script":"0c4011","verification_script":"0c2102ab"}"#,
    )
    .expect("the current v1 raw answer");
    assert_eq!(raw.signature, "11");
    assert_eq!(raw.public_key, "02ab");
    assert_eq!(raw.invocation_script, "0c4011");
}

#[test]
fn a_grant_that_arrives_by_accident_is_the_one_that_lets_nothing_through() {
    assert!(Grant::any().is_any());
    let scoped = Grant::only(["key-2", "key-1", "key-2"]);
    assert!(!scoped.is_any());
    assert_eq!(scoped.key_ids, vec!["key-1", "key-2"]);

    // `{"mode":"only"}` with no list: the service reads that as a grant over no
    // keys at all, and a client that defaulted to `any` would turn a missing field
    // into whole-vault authority.
    let read: Grant = serde_json::from_str(r#"{"mode":"only","key_ids":[]}"#).unwrap();
    assert!(!read.is_any());
    let text: Value = serde_json::to_value(Grant::any()).unwrap();
    assert_eq!(text["mode"], "any");
    assert!(text["key_ids"].as_array().is_some_and(|ids| ids.is_empty()));
}

#[test]
fn the_boundarys_advice_is_carried_and_is_optional() {
    // Two halves of one rule. The advice is the service's judgement, parsed by code
    // so a page can branch on it and by message so a page can print it — and it is
    // `#[serde(default)]`, because a service built before the field exists is a
    // service that answers nothing wrong. Treating an absent field as malformed
    // would turn an additive change on one side of the boundary into a lockstep
    // release on both.
    let written: SavedBoundary = serde_json::from_str(
        r#"{"problems":[{"code":"global-scope-enabled","message":"global witness scope is allowed"}],"policy":{"allow_global_scope":true}}"#,
    )
    .expect("a boundary reply with advice");
    assert_eq!(written.problems.len(), 1);
    assert_eq!(written.problems[0].code, "global-scope-enabled");
    assert_eq!(
        written.problems[0].message, "global witness scope is allowed",
        "the sentence is the service's, not one this crate repeats"
    );
    assert!(written.policy.allow_global_scope);

    let older_service: SavedBoundary =
        serde_json::from_str(r#"{"policy":{"allow_transfer":true}}"#).expect("no advice");
    assert!(older_service.problems.is_empty());

    // The flattened key reply takes the same field, and a blank boundary still
    // means everything closed with no advice to soften it.
    let read: KeyBoundary = serde_json::from_str(
        r#"{"key_id":"key-1","label":"relay","network":"testnet","chain_family":"neox","chain_id":47763,"public_key":"02ab","script_hash":"0x1a2b","address":"NcgY","verification_script":"0c21","signing_enabled":true,"future_key_attestation":{"pcr0":"aa"},"policy":{}}"#,
    )
    .expect("a boundary read without advice");
    assert_eq!(read.key.key_id, "key-1");
    assert_eq!(read.key.chain_family.as_deref(), Some("neox"));
    assert_eq!(read.key.chain_id, Some(47_763));
    assert_eq!(
        read.key.additional_fields["future_key_attestation"]["pcr0"],
        "aa"
    );
    assert!(read.problems.is_empty());
    assert_eq!(read.policy, Policy::default());
    let reencoded = serde_json::to_value(read).expect("the flattened boundary re-encodes");
    assert_eq!(reencoded["future_key_attestation"]["pcr0"], "aa");
    assert!(reencoded["policy"].is_object());
}

#[test]
fn a_refusal_summary_always_names_the_code() {
    let with_message = Refusal {
        code: "consensus-forbidden".to_string(),
        message: "this key may not sign consensus".to_string(),
        status: 403,
    };
    assert_eq!(
        with_message.summary(),
        "consensus-forbidden: this key may not sign consensus"
    );
    let without = Refusal {
        code: "key-not-granted".to_string(),
        message: "   ".to_string(),
        status: 403,
    };
    assert_eq!(without.summary(), "key-not-granted");
}

#[test]
fn only_the_exact_signer_busy_contract_has_a_retry_hint() {
    let refusal = |status, code: &str| Refusal {
        code: code.to_string(),
        message: "retry later".to_string(),
        status,
    };

    assert_eq!(
        refusal(503, "signer-service-busy").retry_after_seconds(),
        Some(1)
    );
    assert_eq!(
        refusal(503, "signer-service-unavailable").retry_after_seconds(),
        None
    );
    assert_eq!(
        refusal(429, "signer-service-busy").retry_after_seconds(),
        None
    );
    assert_eq!(
        refusal(200, "signer-service-busy").retry_after_seconds(),
        None
    );
}

#[test]
fn a_credential_appears_in_one_response_and_in_no_debug() {
    // §5.1: the token is shown once, stored only as a digest, and never logged.
    // The page has to display it; nothing here may print it by accident.
    let created = CreatedCaller {
        caller: crate::signer_client::Caller {
            id: "caller-1".to_string(),
            label: "relayer".to_string(),
            auth_mode: Some("bearer".to_string()),
            workload_public_key: None,
            workload_subject: None,
            key_grant: Grant::any(),
            capabilities: vec!["sign".to_string()],
            allowed_origins: Vec::new(),
            created_at_unix: 1,
            disabled: false,
            additional_fields: Default::default(),
        },
        token: "secret-once-shown".to_string(),
    };
    let text = format!("{created:?}");
    assert!(!text.contains("secret-once-shown"), "{text}");
    assert!(text.contains("<redacted>"), "{text}");
    assert!(text.contains("caller-1"), "the record is still printable");
}

#[test]
fn a_workload_caller_keeps_identity_and_future_metadata() {
    let caller: Caller = serde_json::from_str(
        r#"{"id":"caller-workload","label":"relayer workload","auth_mode":"workload-ed25519","workload_public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","workload_subject":"relayer-prod","key_grant":{"mode":"only","key_ids":["key-1"]},"capabilities":["sign"],"allowed_origins":[],"created_at_unix":7,"disabled":false,"future_attestation":{"pcr0":"bb"}}"#,
    )
    .expect("the current signer workload caller must parse");

    assert_eq!(caller.auth_mode.as_deref(), Some("workload-ed25519"));
    assert_eq!(caller.workload_subject.as_deref(), Some("relayer-prod"));
    assert_eq!(caller.additional_fields["future_attestation"]["pcr0"], "bb");

    let relayed = serde_json::to_value(&caller).expect("the caller relays as JSON");
    assert_eq!(relayed["auth_mode"], "workload-ed25519");
    assert_eq!(relayed["workload_subject"], "relayer-prod");
    assert_eq!(relayed["future_attestation"]["pcr0"], "bb");
}

#[test]
fn an_outcome_is_matched_on_rather_than_unwrapped_into_a_result() {
    // Deliberate: `Outcome::Refused` is not an `Err`, so a `?` on this type would
    // be a policy denial escaping as a transport failure.
    let allowed: Outcome<u8> = Outcome::Allowed(7);
    assert!(allowed.is_allowed());
    assert!(allowed.refusal().is_none());
    assert_eq!(allowed.into_parts().unwrap(), 7);

    let refused: Outcome<u8> = Outcome::Refused(Refusal {
        code: "not-a-signer".to_string(),
        message: String::new(),
        status: 403,
    });
    assert!(!refused.is_allowed());
    assert_eq!(
        refused.refusal().map(|r| r.code.as_str()),
        Some("not-a-signer")
    );
    assert!(refused.into_parts().is_err());
}
