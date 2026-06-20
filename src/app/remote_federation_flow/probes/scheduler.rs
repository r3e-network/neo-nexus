use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn schedule_due_remote_federation_probes(&mut self) {
        self.prune_deleted_remote_federation_runtime_state();
        if !self.remote_federation_monitor_policy.enabled {
            return;
        }

        let now = Instant::now();
        let interval = self.remote_federation_monitor_policy.interval_duration();
        let Some(profile) = self.remote_servers.iter().find_map(|profile| {
            if !profile.enabled || self.remote_federation_pending.contains(&profile.id) {
                return None;
            }
            let due = self
                .remote_federation_last_started
                .get(&profile.id)
                .is_none_or(|last_started| now.duration_since(*last_started) >= interval);
            due.then(|| profile.clone())
        }) else {
            return;
        };

        self.remote_federation_pending.insert(profile.id.clone());
        self.remote_federation_last_started
            .insert(profile.id.clone(), now);
        self.spawn_remote_federation_probe(profile);
    }

    fn spawn_remote_federation_probe(&mut self, profile: RemoteServerProfile) {
        let sender = self.remote_federation_sender.clone();
        let thread_profile = profile.clone();
        if let Err(error) = thread::Builder::new()
            .name(format!("neonexus-remote-fed-{}", thread_profile.id))
            .spawn(move || {
                let report =
                    RemoteFederationClient::probe(&thread_profile, REMOTE_FEDERATION_TIMEOUT)
                        .map_err(|error| error.to_string());
                let _ = sender.send(RemoteFederationProbeResult {
                    profile: thread_profile,
                    report,
                });
            })
        {
            self.remote_federation_pending.remove(&profile.id);
            self.notice = Some(format!(
                "Unable to start remote Federation probe for {}: {error}",
                profile.name
            ));
        }
    }

    fn prune_deleted_remote_federation_runtime_state(&mut self) {
        let live_profile_ids = self
            .remote_servers
            .iter()
            .map(|profile| profile.id.clone())
            .collect::<BTreeSet<_>>();
        self.remote_federation_pending
            .retain(|profile_id| live_profile_ids.contains(profile_id));
        self.remote_federation_last_started
            .retain(|profile_id, _| live_profile_ids.contains(profile_id));
    }
}
