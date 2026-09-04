//! Scoped node-operation credentials for Hermes and other MCP clients.
pub mod hermes_config;
mod model;
pub use model::{AssistantDraft, AssistantProfile};

pub fn connect(
    state: &crate::supervision::EngineState,
    draft: AssistantDraft,
) -> anyhow::Result<(AssistantProfile, String)> {
    state.repository.connect_assistant(draft)
}

pub fn revoke(repository: &crate::repository::Repository, id: &str) -> anyhow::Result<()> {
    repository.revoke_assistant(id)
}

/// Connect an already-configured Hermes profile without changing its channels.
/// Lifecycle and profile mutation stay under the same lock as Start and Stop.
pub fn connect_hermes(
    state: &crate::supervision::EngineState,
    mut draft: AssistantDraft,
    endpoint: &str,
) -> anyhow::Result<AssistantProfile> {
    use anyhow::{bail, Context};
    let supervisor = state
        .supervisor
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if draft.id.is_empty() {
        draft.id = uuid::Uuid::new_v4().to_string();
    }
    if state
        .repository
        .list_assistants()?
        .iter()
        .any(|existing| existing.id == draft.id && existing.agent_id != draft.agent_id)
    {
        bail!("create a separate connection for another Hermes instance, then revoke the previous connection");
    }
    let mut agent = state
        .repository
        .list_agents()?
        .into_iter()
        .find(|agent| agent.profile.id == draft.agent_id)
        .context("Hermes instance not found")?;
    if agent.profile.kind != crate::agents::AgentKind::Hermes {
        bail!("choose a Hermes instance");
    }
    if agent.pid.is_some()
        || agent.desired_running
        || supervisor.is_managing(&agent.profile.process_id())
    {
        bail!("stop Hermes before connecting or changing node permissions");
    }
    let config = agent
        .profile
        .config_path
        .as_deref()
        .context("Hermes configuration path is missing")?;
    let entry_id = format!("neonexus_{}", draft.id);
    hermes_config::validate_entry(config, endpoint, &entry_id)?;
    let (profile, token) = connect(state, draft)?;
    let token = zeroize::Zeroizing::new(token);
    // Encoding the complete ID keeps custom IDs distinct even across case or
    // punctuation differences ("ab-c" must not alias "abc").
    let encoded_id: String = profile
        .id
        .bytes()
        .map(|byte| format!("{byte:02X}"))
        .collect();
    let env_name = format!("NEONEXUS_ASSISTANT_{encoded_id}_TOKEN");
    let report = match hermes_config::inject(config, endpoint, &entry_id, &env_name, &token) {
        Ok(report) => report,
        Err(error) => {
            revoke(&state.repository, &profile.id)
                .context("connection setup failed and its grant could not be revoked")?;
            return Err(error);
        }
    };
    let persist = crate::agents::refresh_config_fingerprint(&mut agent)
        .and_then(|()| state.repository.put_agent(&agent));
    if let Err(error) = persist {
        let revoked = revoke(&state.repository, &profile.id);
        let rolled_back = hermes_config::rollback(&report);
        revoked.context("connection setup failed and its grant could not be revoked")?;
        rolled_back.context("connection setup failed; retained Hermes backups need recovery")?;
        return Err(error)
            .context("connection setup failed; Hermes configuration restored and access revoked");
    }
    Ok(profile)
}
