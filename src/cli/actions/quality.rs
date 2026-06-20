use super::*;

pub(in crate::cli::actions) fn source_purity_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--source-purity")?;
    let report = SourcePurityChecker::check(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: report.to_cli_text(),
    })
}

pub(in crate::cli::actions) fn source_purity_json_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--source-purity-json")?;
    let report = SourcePurityChecker::check(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: source_purity_json_text(&report)?,
    })
}

pub(in crate::cli::actions) fn source_quality_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--source-quality")?;
    let report = SourceQualityChecker::check(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: report.to_cli_text(),
    })
}

pub(in crate::cli::actions) fn source_quality_json_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--source-quality-json")?;
    let report = SourceQualityChecker::check(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: source_quality_json_text(&report)?,
    })
}

pub(in crate::cli::actions) fn native_ui_audit_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--native-ui-audit")?;
    let report = NativeUiAuditor::audit(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: report.to_cli_text(),
    })
}

pub(in crate::cli::actions) fn native_ui_audit_json_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--native-ui-audit-json")?;
    let report = NativeUiAuditor::audit(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: native_ui_audit_json_text(&report)?,
    })
}

pub(in crate::cli::actions) fn ci_policy_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--ci-policy")?;
    let report = CiPolicyChecker::check(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: report.to_cli_text(),
    })
}

pub(in crate::cli::actions) fn ci_policy_json_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--ci-policy-json")?;
    let report = CiPolicyChecker::check(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        exit_code: report.exit_code(),
        text: ci_policy_json_text(&report)?,
    })
}
