use super::*;

use crate::signer_client::{Caller, Grant};

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
