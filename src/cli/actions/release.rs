use super::*;

pub(in crate::cli::actions) fn package_release_text(args: &[String]) -> Result<String> {
    require_arg_count(args, 3, "--package-release")?;
    let package = ReleasePackager::package_current_executable(PathBuf::from(&args[2]))?;
    Ok(package.to_cli_text())
}

pub(in crate::cli::actions) fn verify_release_package_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--verify-release-package")?;
    Ok(
        match ReleasePackageVerifier::verify(PathBuf::from(&args[2])) {
            Ok(verification) => CliAction::PrintWithExitCode {
                text: verification.to_cli_text(),
                exit_code: 0,
            },
            Err(error) => CliAction::PrintWithExitCode {
                text: format!("release-package-verification: failed\nmessage: {error}\n"),
                exit_code: 1,
            },
        },
    )
}

pub(in crate::cli::actions) fn verify_release_package_json_action(
    args: &[String],
) -> Result<CliAction> {
    require_arg_count(args, 3, "--verify-release-package-json")?;
    Ok(
        match ReleasePackageVerifier::verify(PathBuf::from(&args[2])) {
            Ok(verification) => CliAction::PrintWithExitCode {
                text: release_package_verification_json_text(&verification)?,
                exit_code: 0,
            },
            Err(error) => CliAction::PrintWithExitCode {
                text: release_package_verification_failure_json_text(&error.to_string())?,
                exit_code: 1,
            },
        },
    )
}
