use super::*;

pub struct PrivateNetworkLaunchPackVerifier;

impl PrivateNetworkLaunchPackVerifier {
    pub fn validate(path: impl AsRef<Path>) -> Result<PrivateNetworkLaunchPackValidation> {
        let (root_path, manifest_path, manifest) = Self::read_manifest(path)?;

        let mut checks = Vec::new();
        check_schema(&mut checks, &manifest);
        check_scripts(&mut checks, &root_path, &manifest.scripts);
        check_artifacts(&mut checks, &root_path, &manifest.artifacts);
        check_secret_provisioning(&mut checks, &root_path, &manifest);
        check_nodes(&mut checks, &root_path, &manifest);
        check_committee(&mut checks, &root_path, &manifest.committee);

        let passed_count = checks
            .iter()
            .filter(|check| check.status == LaunchPackValidationStatus::Pass)
            .count();
        let warning_count = checks
            .iter()
            .filter(|check| check.status == LaunchPackValidationStatus::Warn)
            .count();
        let failed_count = checks
            .iter()
            .filter(|check| check.status == LaunchPackValidationStatus::Fail)
            .count();

        Ok(PrivateNetworkLaunchPackValidation {
            root_path,
            manifest_path,
            schema_version: manifest.schema_version,
            node_count: manifest.nodes.len(),
            signer_count: manifest.committee.signers.len(),
            passed_count,
            warning_count,
            failed_count,
            checks,
        })
    }

    pub fn sidecar_processes(path: impl AsRef<Path>) -> Result<Vec<CommitteeSidecarProcess>> {
        Ok(Self::sidecar_report(path)?.sidecars)
    }

    pub fn sidecar_report(path: impl AsRef<Path>) -> Result<PrivateNetworkLaunchPackSidecarReport> {
        let (root_path, manifest_path, manifest) = Self::read_manifest(path)?;
        let sidecars = deployment_sidecar_processes(&root_path, &manifest.committee)?;
        Ok(PrivateNetworkLaunchPackSidecarReport {
            root_path,
            manifest_path,
            sidecar_count: sidecars.len(),
            sidecars,
        })
    }

    fn read_manifest(path: impl AsRef<Path>) -> Result<(PathBuf, PathBuf, DeploymentManifest)> {
        let manifest_path = launch_pack_manifest_path(path.as_ref());
        let root_path = manifest_path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        let manifest_text = fs::read_to_string(&manifest_path).with_context(|| {
            format!(
                "failed to read launch pack manifest {}",
                manifest_path.display()
            )
        })?;
        let manifest: DeploymentManifest =
            serde_json::from_str(&manifest_text).context("failed to parse launch pack manifest")?;
        Ok((root_path, manifest_path, manifest))
    }
}
