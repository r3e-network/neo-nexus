use super::super::super::*;

pub(in crate::private_network) fn signer_command_plan_status(
    signer: &DeploymentCommitteeSignerManifest,
    command: &str,
) -> LaunchPackValidationStatus {
    match &signer.signer_command_plan {
        Some(plan)
            if validate_signer_command_plan(plan).is_ok()
                && signer_command_plan_matches_command(plan, command) =>
        {
            LaunchPackValidationStatus::Pass
        }
        Some(_) | None => LaunchPackValidationStatus::Fail,
    }
}
