use std::path::Path;

use super::{read_token_file, resolve_operator_token};

const TOKEN: &str = "c0ffee1234567890c0ffee1234567890c0ffee1234567890c0ffee1234567890";

#[test]
fn protected_token_files_are_the_only_noninteractive_credential_source() {
    let home = tempfile::tempdir().expect("temp directory");
    let path = home.path().join("web.token");
    std::fs::write(&path, format!("{TOKEN}\n")).expect("write token fixture");
    restrict_to_owner(&path);

    let token =
        resolve_operator_token(Some(&path), None, false, false).expect("token file is accepted");
    assert_eq!(token.value.as_str(), TOKEN);
    assert!(!token.generated);
    assert!(resolve_operator_token(None, None, false, false).is_err());
    assert!(resolve_operator_token(Some(&path), None, true, false).is_err());
}

#[test]
fn environment_file_source_is_supported_and_cli_file_takes_precedence() {
    let home = tempfile::tempdir().expect("temp directory");
    let environment_path = home.path().join("environment.token");
    let cli_path = home.path().join("cli.token");
    let environment_token = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee-environment";
    let cli_token = "cccccccccccccccccccccccccccccccc-cli";
    std::fs::write(&environment_path, environment_token).expect("write environment token");
    std::fs::write(&cli_path, cli_token).expect("write CLI token");
    restrict_to_owner(&environment_path);
    restrict_to_owner(&cli_path);

    let from_environment = resolve_operator_token(None, Some(&environment_path), false, false)
        .expect("environment file is accepted");
    assert_eq!(from_environment.value.as_str(), environment_token);

    let from_cli = resolve_operator_token(Some(&cli_path), Some(&environment_path), false, false)
        .expect("CLI file is accepted");
    assert_eq!(from_cli.value.as_str(), cli_token);
}

#[test]
fn interactive_bootstrap_is_generated_only_for_a_terminal() {
    let token = resolve_operator_token(None, None, false, true).expect("interactive bootstrap");
    assert!(token.generated);
    assert!(token.value.len() >= 32);
}

#[test]
fn token_file_rejects_short_or_oversized_values() {
    let home = tempfile::tempdir().expect("temp directory");
    let short = home.path().join("short.token");
    std::fs::write(&short, "too-short").unwrap();
    restrict_to_owner(&short);
    assert!(read_token_file(&short).is_err());

    let oversized = home.path().join("oversized.token");
    std::fs::write(&oversized, "x".repeat(4 * 1024 + 1)).unwrap();
    restrict_to_owner(&oversized);
    assert!(read_token_file(&oversized).is_err());
}

#[cfg(unix)]
#[test]
fn token_file_rejects_group_or_world_access() {
    use std::os::unix::fs::PermissionsExt;

    let home = tempfile::tempdir().expect("temp directory");
    let path = home.path().join("shared.token");
    std::fs::write(&path, TOKEN).expect("write token fixture");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
    assert!(read_token_file(&path).is_err());
}

#[cfg(unix)]
fn restrict_to_owner(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

#[cfg(not(unix))]
fn restrict_to_owner(_path: &Path) {}
