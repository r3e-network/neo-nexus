use super::*;

use crate::signer_client::{Caller, Grant};

#[test]
fn every_policy_field_reads_its_own_slot() {
    // The key page draws every control from a field constant and asks the draft
    // for its value by that same constant. So the only way a control can show
    // the wrong text is if this mapping is wrong — and a form showing one field's
    // text under another field's label is worse than an empty one, because
    // nothing about the page looks wrong while the operator edits it.
    use crate::web::form_state::FormDraft;
    use crate::web::signer_form::{self, PolicyForm};

    let form = PolicyForm {
        chain_family: signer_form::CHAIN_FAMILY.to_string(),
        allow_consensus: signer_form::ALLOW_CONSENSUS.to_string(),
        allow_raw: signer_form::ALLOW_RAW.to_string(),
        allow_transfer: signer_form::ALLOW_TRANSFER.to_string(),
        allow_contract_call: signer_form::ALLOW_CONTRACT_CALL.to_string(),
        allow_global_scope: signer_form::ALLOW_GLOBAL_SCOPE.to_string(),
        contract_whitelist: signer_form::CONTRACT_WHITELIST.to_string(),
        contract_blacklist: signer_form::CONTRACT_BLACKLIST.to_string(),
        contract_method_whitelist: signer_form::CONTRACT_METHOD_WHITELIST.to_string(),
        contract_method_blacklist: signer_form::CONTRACT_METHOD_BLACKLIST.to_string(),
        asset_whitelist: signer_form::ASSET_WHITELIST.to_string(),
        asset_blacklist: signer_form::ASSET_BLACKLIST.to_string(),
        asset_limits: signer_form::ASSET_LIMITS.to_string(),
        transfer_to_whitelist: signer_form::TRANSFER_TO_WHITELIST.to_string(),
        transfer_to_blacklist: signer_form::TRANSFER_TO_BLACKLIST.to_string(),
        max_single_amount: signer_form::MAX_SINGLE_AMOUNT.to_string(),
        window_seconds: signer_form::WINDOW_SECONDS.to_string(),
        window_max_amount: signer_form::WINDOW_MAX_AMOUNT.to_string(),
        max_signers: signer_form::MAX_SIGNERS.to_string(),
        max_system_fee: signer_form::MAX_SYSTEM_FEE.to_string(),
        max_network_fee: signer_form::MAX_NETWORK_FEE.to_string(),
        signature_window_seconds: signer_form::SIGNATURE_WINDOW_SECONDS.to_string(),
        signature_window_count: signer_form::SIGNATURE_WINDOW_COUNT.to_string(),
        evm_max_gas_price: signer_form::EVM_MAX_GAS_PRICE.to_string(),
        evm_max_gas_limit: signer_form::EVM_MAX_GAS_LIMIT.to_string(),
        evm_method_whitelist: signer_form::EVM_METHOD_WHITELIST.to_string(),
        evm_method_blacklist: signer_form::EVM_METHOD_BLACKLIST.to_string(),
        evm_chain_id: signer_form::EVM_CHAIN_ID.to_string(),
        additional_fields: signer_form::ADDITIONAL_FIELDS.to_string(),
    };
    for name in signer_form::POLICY_FIELDS {
        assert_eq!(
            form.value(name),
            name,
            "the control named {name} read another field's text"
        );
    }
    let unique = signer_form::POLICY_FIELDS
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        unique.len(),
        signer_form::POLICY_FIELDS.len(),
        "a field name is listed twice, so one control is never checked"
    );
}

#[test]
fn signer_tabs_default_closed_and_accept_only_known_sections() {
    assert_eq!(SignerTab::from_query(None), SignerTab::Overview);
    assert_eq!(SignerTab::from_query(Some("tab=keys")), SignerTab::Keys);
    assert_eq!(
        SignerTab::from_query(Some("flash=done&tab=callers")),
        SignerTab::Callers
    );
    assert_eq!(SignerTab::from_query(Some("tab=audit")), SignerTab::Audit);
    assert_eq!(
        SignerTab::from_query(Some("tab=private-keys")),
        SignerTab::Overview
    );
}

#[test]
fn signer_tab_navigation_marks_one_current_section() {
    let markup = tabs(SignerTab::Callers);
    assert_eq!(markup.matches(r#"aria-current="page""#).count(), 1);
    for tab in ["overview", "keys", "callers", "audit"] {
        assert!(markup.contains(&format!("/signer?tab={tab}")), "{markup}");
    }
}

#[test]
fn caller_forms_use_document_unique_ids() {
    let markup = format!(
        "{}{}",
        overview::new_caller_form(&[]),
        overview::new_workload_caller_form(&[])
    );
    let ids = markup
        .split(r#" id=""#)
        .skip(1)
        .filter_map(|tail| tail.split('"').next())
        .collect::<Vec<_>>();
    let unique = ids
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(ids.len(), unique.len(), "duplicate ids in {markup}");
    assert!(unique.contains("bearer-caller-label"));
    assert!(unique.contains("workload-caller-label"));
}

#[test]
fn caller_rotation_and_deletion_require_review_pages() {
    let caller = Caller {
        id: "relay-primary".to_string(),
        label: "Primary relay".to_string(),
        auth_mode: Some("bearer".to_string()),
        workload_public_key: None,
        workload_subject: None,
        key_grant: Grant::any(),
        capabilities: vec!["sign".to_string()],
        allowed_origins: Vec::new(),
        created_at_unix: 1,
        disabled: false,
        additional_fields: Default::default(),
    };
    let rotate = caller::confirmation_body("", &caller, caller::CallerAction::Rotate);
    let delete = caller::confirmation_body("", &caller, caller::CallerAction::Delete);
    assert!(rotate.contains(r#"action="/signer/callers/relay-primary/rotate""#));
    assert!(rotate.contains("current bearer token stops working immediately"));
    assert!(delete.contains(r#"action="/signer/callers/relay-primary/delete""#));
    assert!(delete.contains("permanently revokes the caller"));
    assert!(rotate.contains(r#"class="danger""#));
    assert!(delete.contains(r#"class="danger""#));
}
