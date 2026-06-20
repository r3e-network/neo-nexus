use super::*;

pub(in crate::cli::actions) fn validate_wallet_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--validate-wallet")?;
    let report = NeoWalletValidator::validate_path(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: if report.is_success() { 0 } else { 1 },
        text: report.to_cli_text(),
    })
}

pub(in crate::cli::actions) fn validate_wallet_json_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--validate-wallet-json")?;
    let report = NeoWalletValidator::validate_path(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: if report.is_success() { 0 } else { 1 },
        text: wallet_validation_json_text(&report)?,
    })
}

pub(in crate::cli::actions) fn import_wallet_profile_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 6, "--import-wallet-profile")?;
    let repository = Repository::open(PathBuf::from(&args[2]))
        .with_context(|| format!("failed to open workspace database {}", args[2]))?;
    let profile =
        NeoWalletValidator::profile_from_path(&args[3], &args[4], &args[5], current_unix_time()?)?;
    repository.upsert_neo_wallet_profile(&profile)?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: 0,
        text: wallet_profile_import_text(&profile),
    })
}
