use super::super::*;

mod channels;
mod initial_state;
mod policies;
mod recovery;
mod workspace_prefs;

use self::{
    channels::StartupChannels, initial_state::build_initial_app, policies::StartupPolicies,
    recovery::recover_transient_runtime_state, workspace_prefs::StartupWorkspacePrefs,
};

impl NeoNexusApp {
    pub fn open_default() -> anyhow::Result<Self> {
        let data_dir = data_dir();
        let repository = Repository::open(data_dir.join("neonexus.db"))?;
        Ok(Self::new(repository))
    }

    pub fn new(repository: Repository) -> Self {
        let channels = StartupChannels::open();
        let policies = StartupPolicies::load(&repository);
        let prefs = StartupWorkspacePrefs::load(&repository);
        let mut app = build_initial_app(repository, policies, prefs, channels);
        recover_transient_runtime_state(&mut app);
        app.reload_workspace_data();
        app
    }
}
