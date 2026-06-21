use std::path::{Path, PathBuf};

use super::{lookup::signer_path_candidate_names_from_pathext, signer_sidecar_process_binary_path};

#[test]
fn sidecar_process_binary_path_keeps_path_lookup_commands_unresolved() {
    let root = Path::new("/tmp/private-pack");

    assert_eq!(
        signer_sidecar_process_binary_path(root, "neo-signer"),
        PathBuf::from("neo-signer")
    );
    assert_eq!(
        signer_sidecar_process_binary_path(root, "signer-bin/neo-signer"),
        root.join("signer-bin").join("neo-signer")
    );
}

#[test]
fn windows_candidate_names_append_pathext_for_extensionless_binary() {
    let names = signer_path_candidate_names_from_pathext(Path::new("neo-signer"), ".EXE;.CMD");

    assert_eq!(
        names,
        [
            PathBuf::from("neo-signer"),
            PathBuf::from("neo-signer.EXE"),
            PathBuf::from("neo-signer.CMD"),
        ]
    );
}

#[test]
fn windows_candidate_names_keep_explicit_extension() {
    let names = signer_path_candidate_names_from_pathext(Path::new("neo-signer.exe"), ".EXE;.CMD");

    assert_eq!(names, [PathBuf::from("neo-signer.exe")]);
}
