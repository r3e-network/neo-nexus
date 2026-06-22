use super::*;

pub(in crate::cli) fn action_from_args_vec(args: &[String]) -> Result<CliAction> {
    let Some(first) = args.get(1) else {
        anyhow::bail!("missing NeoNexus CLI option; run without CLI dispatch to start the GUI");
    };

    match first.as_str() {
        "-V" | "--version" => {
            require_arg_count(args, 2, "--version")?;
            Ok(CliAction::Print(version_text()))
        }
        "-h" | "--help" => {
            require_arg_count(args, 2, "--help")?;
            Ok(CliAction::Print(help_text()))
        }
        "--self-check" => {
            require_arg_count(args, 2, "--self-check")?;
            Ok(CliAction::Print(self_check_text()?))
        }
        "--completions" => completions_action(args),
        "--runtime-smoke" => Ok(CliAction::Print(runtime_smoke_text(args)?)),
        "--runtime-smoke-json" => runtime_smoke_json_action(args),
        "--rpc-health" => Ok(CliAction::Print(rpc_health_text(args)?)),
        "--rpc-health-json" => rpc_health_json_action(args),
        "--workspace-readiness" => workspace_readiness_action(args),
        "--workspace-readiness-json" => workspace_readiness_json_action(args),
        "--workspace-metrics" => workspace_metrics_action(args),
        "--workspace-metrics-json" => workspace_metrics_json_action(args),
        "--workspace-metrics-prometheus" => workspace_metrics_prometheus_action(args),
        "--workspace-integrity" => workspace_integrity_action(args),
        "--workspace-integrity-json" => workspace_integrity_json_action(args),
        "--source-purity" => source_purity_action(args),
        "--source-purity-json" => source_purity_json_action(args),
        "--source-quality" => source_quality_action(args),
        "--source-quality-json" => source_quality_json_action(args),
        "--native-ui-audit" => native_ui_audit_action(args),
        "--native-ui-audit-json" => native_ui_audit_json_action(args),
        "--ci-policy" => ci_policy_action(args),
        "--ci-policy-json" => ci_policy_json_action(args),
        "--alert-preview" => Ok(CliAction::Print(alert_preview_text(args)?)),
        "--alert-preview-json" => alert_preview_json_action(args),
        "--export-readiness-report" => workspace_readiness_report_action(args),
        "--export-support-bundle" => Ok(CliAction::Print(export_support_bundle_text(args)?)),
        "--export-support-bundle-json" => {
            Ok(CliAction::Print(export_support_bundle_json_text(args)?))
        }
        "--export-event-journal" => Ok(CliAction::Print(export_event_journal_text(args)?)),
        "--export-node-configs" => Ok(CliAction::Print(export_node_configs_text(args)?)),
        "--export-node-configs-json" => Ok(CliAction::Print(export_node_configs_json_text(args)?)),
        "--generate-node-config" => generate_node_config_action(args),
        "--generate-node-config-json" => generate_node_config_json_action(args),
        "--validate-node-config" => validate_node_config_action(args),
        "--validate-node-config-json" => validate_node_config_json_action(args),
        "--export-backup" => Ok(CliAction::Print(export_backup_text(args)?)),
        "--export-backup-json" => Ok(CliAction::Print(export_backup_json_text(args)?)),
        "--import-backup" => Ok(CliAction::Print(import_backup_text(args)?)),
        "--import-backup-json" => Ok(CliAction::Print(import_backup_json_text(args)?)),
        "--validate-backup" => Ok(CliAction::Print(validate_backup_text(args)?)),
        "--validate-backup-json" => Ok(CliAction::Print(validate_backup_json_text(args)?)),
        "--validate-wallet" => validate_wallet_action(args),
        "--validate-wallet-json" => validate_wallet_json_action(args),
        "--import-wallet-profile" => import_wallet_profile_action(args),
        "--validate-launch-pack" => validate_launch_pack_action(args),
        "--launch-pack-sidecars" => launch_pack_sidecars_action(args),
        "--launch-pack-sidecars-json" => launch_pack_sidecars_json_action(args),
        "--package-release" => Ok(CliAction::Print(package_release_text(args)?)),
        "--verify-release-package" => verify_release_package_action(args),
        "--verify-release-package-json" => verify_release_package_json_action(args),
        option => match suggest::suggest_option(option) {
            Some(suggestion) => {
                anyhow::bail!("unsupported NeoNexus option: {option}; did you mean {suggestion}?")
            }
            None => anyhow::bail!("unsupported NeoNexus option: {option}"),
        },
    }
}
