use super::*;

pub(in crate::private_network) fn check_scripts(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    scripts: &DeploymentScriptsManifest,
) {
    for (label, script_path) in [
        ("runbook", &scripts.runbook),
        ("preflight-unix", &scripts.preflight_unix),
        ("preflight-windows", &scripts.preflight_windows),
        ("health-unix", &scripts.health_unix),
        ("health-windows", &scripts.health_windows),
        ("start-unix", &scripts.start_unix),
        ("stop-unix", &scripts.stop_unix),
        ("start-windows", &scripts.start_windows),
        ("stop-windows", &scripts.stop_windows),
    ] {
        let Some(path) = safe_launch_pack_child(root_path, script_path) else {
            add_check(
                checks,
                "scripts",
                label,
                LaunchPackValidationStatus::Fail,
                format!("script path escapes launch pack: {script_path}"),
            );
            continue;
        };
        add_file_check(checks, "scripts", label, &path);
    }
}
