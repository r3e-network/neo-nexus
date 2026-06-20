use super::*;

pub(super) struct LaunchPackWriteContext {
    pub(super) generated_at_unix: u64,
    pub(super) network_magic: u32,
    pub(super) validators_count: u8,
    pub(super) seed_nodes: Vec<String>,
    pub(super) committee_public_keys: Vec<String>,
    pub(super) root_path: PathBuf,
}

impl LaunchPackWriteContext {
    pub(super) fn prepare(request: &PrivateNetworkDeploymentRequest) -> Result<Self> {
        let generated_at_unix = current_unix_time()?;
        let network_magic = deployment_network_magic(request.plan.template, request.plan.node_type);
        let validators_count = request.plan.consensus_count() as u8;
        let seed_nodes = seed_nodes(&request.plan);
        let committee_public_keys = request
            .committee
            .as_ref()
            .map_or_else(Vec::new, CommitteeRoster::public_keys);
        let root_path = request.output_dir.join(deployment_slug(
            request.plan.template,
            request.plan.node_type,
            network_magic,
        ));
        fs::create_dir_all(&root_path).with_context(|| {
            format!(
                "failed to create private network directory {}",
                root_path.display()
            )
        })?;

        Ok(Self {
            generated_at_unix,
            network_magic,
            validators_count,
            seed_nodes,
            committee_public_keys,
            root_path,
        })
    }
}
