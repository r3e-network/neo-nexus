use super::*;

pub(in crate::private_network) fn check_artifacts(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    artifacts: &[DeploymentArtifactManifest],
) {
    if artifacts.is_empty() {
        add_check(
            checks,
            "artifact-integrity",
            "inventory",
            LaunchPackValidationStatus::Fail,
            "manifest has no generated artifact integrity inventory".to_string(),
        );
        return;
    }

    add_check(
        checks,
        "artifact-integrity",
        "inventory",
        LaunchPackValidationStatus::Pass,
        format!("{} generated artifacts", artifacts.len()),
    );

    for artifact in artifacts {
        let label = artifact.label.trim();
        if label.is_empty() {
            add_check(
                checks,
                "artifact-integrity",
                "artifact",
                LaunchPackValidationStatus::Fail,
                "artifact label is empty".to_string(),
            );
            continue;
        }

        let Some(path) = safe_launch_pack_child(root_path, &artifact.path) else {
            add_check(
                checks,
                "artifact-integrity",
                label,
                LaunchPackValidationStatus::Fail,
                format!("artifact path escapes launch pack: {}", artifact.path),
            );
            continue;
        };

        let expected = match normalize_sha256(&artifact.sha256) {
            Ok(expected) => expected,
            Err(_) => {
                add_check(
                    checks,
                    "artifact-integrity",
                    format!("{label} sha256"),
                    LaunchPackValidationStatus::Fail,
                    "manifest artifact sha256 is missing or invalid".to_string(),
                );
                continue;
            }
        };

        let (actual, actual_bytes) = match sha256_file(&path) {
            Ok(result) => result,
            Err(error) => {
                add_check(
                    checks,
                    "artifact-integrity",
                    format!("{label} sha256"),
                    LaunchPackValidationStatus::Fail,
                    format!("unable to hash artifact: {error}"),
                );
                continue;
            }
        };

        let status = if actual == expected && actual_bytes == artifact.bytes {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        };
        add_check(
            checks,
            "artifact-integrity",
            format!("{label} sha256"),
            status,
            format!(
                "expected {expected} / {} bytes, actual {actual} / {actual_bytes} bytes",
                artifact.bytes
            ),
        );
    }
}
