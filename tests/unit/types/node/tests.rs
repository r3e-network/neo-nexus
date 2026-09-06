use super::{node_workspace_path, validate_node_id};

#[test]
fn node_id_grammar_accepts_generated_shape_and_rejects_path_syntax() {
    for valid in ["node-1234", "A", "validator_01", "9-node"] {
        validate_node_id(valid).unwrap();
    }
    for invalid in [
        "",
        "-node",
        "_node",
        ".",
        "..",
        "../outside",
        "..\\outside",
        "/absolute",
        "C:\\outside",
        "nested/node",
        "node space",
        "node-非ascii",
    ] {
        assert!(validate_node_id(invalid).is_err(), "accepted {invalid:?}");
    }
    assert!(validate_node_id(&"a".repeat(129)).is_err());
}

#[test]
fn node_workspace_path_is_contained_below_root() {
    let root = std::path::PathBuf::from("workspace").join("nodes");
    let safe = node_workspace_path(&root, "node-1234").unwrap();
    assert_eq!(safe, root.join("node-1234"));
    assert!(safe.starts_with(&root));

    for unsafe_id in ["../outside", "..\\outside", "/absolute", "C:\\outside"] {
        assert!(node_workspace_path(&root, unsafe_id).is_err());
    }
}
