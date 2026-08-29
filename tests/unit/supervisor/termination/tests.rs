//! Identity matching for pid-based stops. Getting this wrong means signalling a
//! process that has nothing to do with the node, so the refusals matter as much
//! as the matches.

use std::path::Path;

use crate::supervisor::termination::name_matches_binary;

fn matches(reported: &str, binary: &str) -> bool {
    name_matches_binary(reported, Path::new(binary))
}

#[test]
fn an_exact_name_matches() {
    assert!(matches("neo-go", "/usr/local/bin/neo-go"));
    assert!(matches("neo-go", "/opt/neo/neo-go"));
}

#[test]
fn the_windows_extension_is_tolerated_on_both_sides() {
    assert!(matches("neo-go.exe", r"C:\neo\neo-go.exe"));
    assert!(matches("neo-go", r"C:\neo\neo-go.exe"));
    assert!(matches("neo-go.exe", r"C:\neo\neo-go"));
}

#[test]
fn comparison_ignores_case() {
    assert!(matches("Neo-Go", "/opt/neo/neo-go"));
    assert!(matches("neo-go", "/opt/neo/NEO-GO"));
}

#[test]
fn a_different_program_never_matches() {
    // The recycled-pid case this check exists for.
    assert!(!matches("postgres", "/opt/neo/neo-go"));
    assert!(!matches("sleep", "ping.exe"));
    // A prefix is not an identity.
    assert!(!matches("neo-gosh", "/opt/neo/neo-go"));
}

#[test]
fn an_unusable_recorded_path_is_refused() {
    assert!(!matches("anything", ""));
    assert!(!matches("anything", "/"));
}

#[test]
fn surrounding_whitespace_in_the_reported_name_is_ignored() {
    assert!(matches("  neo-go  ", "/opt/neo/neo-go"));
}
