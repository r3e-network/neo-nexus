//! What the runtime-catalogue form refuses, and which control it blames.
//!
//! The form is six fields and one of them is a size limit that has to parse.
//! Attribution is the value of the redraw: coming back with the text intact but
//! the complaint in a banner above six controls saves a copy-paste, not a
//! search.

use crate::web::{
    catalog_form::{
        build, CatalogDraft, CATALOG_FIELDS, CATALOG_ID, CATALOG_LABEL, CATALOG_MAX_BYTES,
        CATALOG_PUBLIC_KEY, CATALOG_SIGNATURE_SOURCE, CATALOG_SOURCE,
    },
    form_state::FormDraft,
};

/// A catalogue the workspace would accept, so a case only has to say what it
/// changes.
fn draft() -> CatalogDraft {
    CatalogDraft {
        id: "neo-official".to_string(),
        label: "Neo official releases".to_string(),
        source: "https://catalogs.example.test/neo.json".to_string(),
        signature_source: "https://catalogs.example.test/neo.json.sig".to_string(),
        ed25519_public_key: "A".repeat(44),
        max_bytes: String::new(),
    }
}

/// Put a value in the slot a field constant names.
fn set(draft: &mut CatalogDraft, field: &str, value: &str) {
    match field {
        CATALOG_ID => draft.id = value.to_string(),
        CATALOG_LABEL => draft.label = value.to_string(),
        CATALOG_SOURCE => draft.source = value.to_string(),
        CATALOG_SIGNATURE_SOURCE => draft.signature_source = value.to_string(),
        CATALOG_PUBLIC_KEY => draft.ed25519_public_key = value.to_string(),
        CATALOG_MAX_BYTES => draft.max_bytes = value.to_string(),
        // A name the form does not have: the caller is the table below, which
        // only ever passes a constant from this module.
        _ => {}
    }
}

#[test]
fn a_refused_catalogue_names_the_field_the_operator_has_to_change() {
    let cases: [(&str, &str, &str); 4] = [
        (CATALOG_ID, "   ", CATALOG_ID),
        (CATALOG_LABEL, "", CATALOG_LABEL),
        (CATALOG_SOURCE, " ", CATALOG_SOURCE),
        (CATALOG_MAX_BYTES, "four megabytes", CATALOG_MAX_BYTES),
    ];
    for (field, value, blamed) in cases {
        let mut input = draft();
        set(&mut input, field, value);
        let refusal = build(&input).expect_err(&format!("{field}={value:?} should be refused"));
        assert_eq!(
            refusal.field, blamed,
            "{field}={value:?} was blamed on {} instead of {blamed}: {}",
            refusal.field, refusal.message
        );
        assert!(
            !refusal.message.is_empty(),
            "{field} was refused with no reason"
        );
    }
}

#[test]
fn a_catalogue_that_is_accepted_is_trimmed_and_defaulted() {
    let mut input = draft();
    input.id = "  neo-official  ".to_string();
    input.label = " Neo official releases ".to_string();
    input.source = " https://catalogs.example.test/neo.json ".to_string();
    input.max_bytes = "   ".to_string();

    let built = build(&input).expect("a blank size limit should fall back to the default");
    assert_eq!(built.id, "neo-official");
    assert_eq!(built.label, "Neo official releases");
    assert_eq!(built.source, "https://catalogs.example.test/neo.json");
    assert!(
        built.max_bytes > 0,
        "a blank limit must not become zero, which the domain refuses"
    );
    // The optional pair travels as given; whether it is a usable pair is the
    // domain's call, and this test must not pretend otherwise.
    assert!(built.signature_source.is_some());
    assert!(built.ed25519_public_key.is_some());
}

#[test]
fn a_blank_optional_field_is_no_value_rather_than_an_empty_string() {
    let mut input = draft();
    input.signature_source = "  ".to_string();
    input.ed25519_public_key = String::new();

    let built = build(&input).expect("the optional pair may be absent");
    assert_eq!(built.signature_source, None);
    assert_eq!(built.ed25519_public_key, None);
}

#[test]
fn every_field_reads_its_own_slot() {
    let mut input = CatalogDraft::default();
    for field in CATALOG_FIELDS {
        set(&mut input, field, field);
    }
    for field in CATALOG_FIELDS {
        assert_eq!(
            input.value(field),
            field,
            "the slot for {field} holds another field's value"
        );
    }

    let mut seen: Vec<&str> = CATALOG_FIELDS.to_vec();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(
        seen.len(),
        CATALOG_FIELDS.len(),
        "two fields share a name: {CATALOG_FIELDS:?}"
    );
}
