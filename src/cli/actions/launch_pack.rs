use super::*;

pub(in crate::cli::actions) fn validate_launch_pack_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--validate-launch-pack")?;
    let report = PrivateNetworkLaunchPackVerifier::validate(PathBuf::from(&args[2]))?;
    let validation_succeeded = report.is_success();
    let mut report_text = report.to_cli_text();
    let report_write_succeeded = match report.write_reports() {
        Ok(paths) => {
            report_text.push_str(&format!(
                "report-text: {}\nreport-json: {}\n",
                paths.text_path.display(),
                paths.json_path.display()
            ));
            true
        }
        Err(error) => {
            report_text.push_str(&format!("report-write: failed: {error}\n"));
            false
        }
    };
    let exit_code = if validation_succeeded && report_write_succeeded {
        0
    } else {
        1
    };
    Ok(CliAction::PrintWithExitCode {
        text: report_text,
        exit_code,
    })
}

pub(in crate::cli::actions) fn launch_pack_sidecars_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--launch-pack-sidecars")?;
    let report = PrivateNetworkLaunchPackVerifier::sidecar_report(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        text: report.to_cli_text(),
        exit_code: 0,
    })
}

pub(in crate::cli::actions) fn launch_pack_sidecars_json_action(
    args: &[String],
) -> Result<CliAction> {
    require_arg_count(args, 3, "--launch-pack-sidecars-json")?;
    let report = PrivateNetworkLaunchPackVerifier::sidecar_report(PathBuf::from(&args[2]))?;
    Ok(CliAction::PrintWithExitCode {
        text: launch_pack_sidecars_json_text(&report)?,
        exit_code: 0,
    })
}
