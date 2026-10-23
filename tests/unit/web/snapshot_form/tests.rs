//! What the snapshot register form refuses, and which control it blames.
//!
//! The register form is nine fields wide and one of them is a 64-character
//! digest. Attribution is the whole value of the redraw: a page that comes back
//! with the text intact but the complaint in a banner above nine controls has
//! only saved the operator a copy-paste, not a search.

use crate::web::{
    form_state::{FormDraft, GENERAL},
    snapshot_form::{
        build, SnapshotDraft, DOWNLOAD_FILE_NAME, DOWNLOAD_MAX_BYTES, EXPECTED_SHA256, ID, LABEL,
        NETWORK, NODE_TYPE, SNAPSHOT_FIELDS, SOURCE_PATH, SOURCE_URL,
    },
};

/// A manifest the workspace would accept, so a case only has to say what it
/// changes.
fn draft() -> SnapshotDraft {
    SnapshotDraft {
        id: "genesis-snap".to_string(),
        label: "Genesis snapshot".to_string(),
        network: "testnet".to_string(),
        node_type: "neo-rs".to_string(),
        source_path: "/archive/chain.acc".to_string(),
        source_url: String::new(),
        download_file_name: String::new(),
        download_max_bytes: String::new(),
        expected_sha256: "a".repeat(64),
    }
}

/// Put a value in the slot a field constant names.
fn set(draft: &mut SnapshotDraft, field: &str, value: &str) {
    match field {
        ID => draft.id = value.to_string(),
        LABEL => draft.label = value.to_string(),
        NETWORK => draft.network = value.to_string(),
        NODE_TYPE => draft.node_type = value.to_string(),
        SOURCE_PATH => draft.source_path = value.to_string(),
        SOURCE_URL => draft.source_url = value.to_string(),
        DOWNLOAD_FILE_NAME => draft.download_file_name = value.to_string(),
        DOWNLOAD_MAX_BYTES => draft.download_max_bytes = value.to_string(),
        EXPECTED_SHA256 => draft.expected_sha256 = value.to_string(),
        // A name the form does not have: the caller is the table below, which
        // only ever passes a constant from this module.
        _ => {}
    }
}

#[test]
fn a_refused_manifest_names_the_field_the_operator_has_to_change() {
    let cases: [(&str, &str, &str); 7] = [
        (ID, "   ", ID),
        (LABEL, "", LABEL),
        (NETWORK, "polygon", NETWORK),
        (NODE_TYPE, "geth", NODE_TYPE),
        (DOWNLOAD_MAX_BYTES, "eight gigabytes", DOWNLOAD_MAX_BYTES),
        (EXPECTED_SHA256, "deadbeef", EXPECTED_SHA256),
        // Two halves of one requirement, and neither was supplied: the pair is
        // what failed, so the complaint belongs above the form rather than
        // beside a control that is not individually wrong.
        (SOURCE_PATH, "", GENERAL),
    ];
    for (field, value, blamed) in cases {
        let mut input = draft();
        set(&mut input, field, value);
        if field == SOURCE_PATH {
            input.source_url = String::new();
        }
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
fn a_manifest_that_is_accepted_is_trimmed_and_defaulted() {
    let mut input = draft();
    input.id = "  genesis-snap  ".to_string();
    input.label = " Genesis snapshot ".to_string();
    input.source_path = " /archive/chain.acc ".to_string();
    input.download_max_bytes = "  ".to_string();

    let built = build(&input).expect("a blank size limit should fall back to the default");
    assert_eq!(built.id, "genesis-snap");
    assert_eq!(built.label, "Genesis snapshot");
    assert_eq!(
        built.source_path,
        std::path::Path::new("/archive/chain.acc")
    );
    assert!(
        built.download_max_bytes > 0,
        "a blank limit must not become zero, which would refuse every download"
    );
    assert_eq!(built.source_url, None);
    // The digest is carried as typed; normalising it is the workspace's job.
    assert_eq!(built.expected_sha256, "a".repeat(64));
}

#[test]
fn a_url_alone_is_a_source() {
    let mut input = draft();
    input.source_path = String::new();
    input.source_url = " https://example.test/chain.acc ".to_string();

    let built = build(&input).expect("a URL is a source on its own");
    assert_eq!(built.source_path, std::path::Path::new(""));
    assert_eq!(
        built.source_url.as_deref(),
        Some("https://example.test/chain.acc")
    );
}

#[test]
fn every_field_reads_its_own_slot() {
    // A sentinel per slot: if two arms of `value` pointed at the same field, the
    // control for one would draw the other's text.
    let mut input = SnapshotDraft::default();
    for field in SNAPSHOT_FIELDS {
        set(&mut input, field, field);
    }
    for field in SNAPSHOT_FIELDS {
        assert_eq!(
            input.value(field),
            field,
            "the slot for {field} holds another field's value"
        );
    }

    let mut seen: Vec<&str> = SNAPSHOT_FIELDS.to_vec();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(
        seen.len(),
        SNAPSHOT_FIELDS.len(),
        "two fields share a name: {SNAPSHOT_FIELDS:?}"
    );
}
