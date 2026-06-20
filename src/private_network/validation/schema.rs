use super::*;

pub(in crate::private_network) fn check_schema(
    checks: &mut Vec<LaunchPackValidationCheck>,
    manifest: &DeploymentManifest,
) {
    add_check(
        checks,
        "schema",
        "version",
        if manifest.schema_version == LAUNCH_PACK_SCHEMA_VERSION {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        format!(
            "manifest schema version {}; expected {}",
            manifest.schema_version, LAUNCH_PACK_SCHEMA_VERSION
        ),
    );
    add_check(
        checks,
        "schema",
        "network",
        if manifest.network == Network::Private.to_string() {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        format!("network {}", manifest.network),
    );
    add_check(
        checks,
        "schema",
        "validators",
        if manifest.validators_count > 0 {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        format!("{} validators", manifest.validators_count),
    );
}
