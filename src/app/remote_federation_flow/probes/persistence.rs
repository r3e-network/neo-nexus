use super::*;

impl NeoNexusApp {
    pub(super) fn persist_remote_server_probe(
        &mut self,
        report: &RemoteServerProbeReport,
    ) -> anyhow::Result<RemoteServerProbeRecord> {
        let record = self.repository.record_remote_server_probe(report)?;
        self.repository
            .prune_remote_server_probes_keep_recent_per_profile(REMOTE_PROBE_RETAIN_PER_PROFILE)?;
        Ok(record)
    }
}
