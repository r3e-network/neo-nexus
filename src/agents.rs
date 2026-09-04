//! Operator-supplied Hermes, signer and node-management companion processes.
//! Profiles contain executable/configuration references, never custody keys.
mod hermes;
mod hermes_runtime;
mod lifecycle;
mod monitor;
mod profile;

pub use lifecycle::{delete, forget_stale, save, start, stop};
pub use monitor::{observe_exit, tick};
pub use profile::{AgentKind, AgentProfile, AgentRecord, AgentStatus};

pub(crate) fn refresh_config_fingerprint(record: &mut AgentRecord) -> anyhow::Result<()> {
    record.profile.config_sha256 = record
        .profile
        .config_path
        .as_deref()
        .map(profile::digest)
        .transpose()?;
    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/agents/tests.rs"]
mod tests;
