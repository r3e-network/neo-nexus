use crate::backup::schema::WorkspaceBackup;
use anyhow::{bail, Result};
use std::{collections::BTreeSet, path::Path};

pub(super) fn validate_agents(backup: &WorkspaceBackup) -> Result<()> {
    if backup.schema_version < 8 && !backup.agents.is_empty() {
        bail!("agent profiles require backup schema version 8");
    }
    let node_ids: BTreeSet<_> = backup.nodes.iter().map(|node| node.id.as_str()).collect();
    let mut ids = BTreeSet::new();
    for profile in &backup.agents {
        if profile.id.is_empty()
            || !profile
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            bail!("agent id must contain only letters, numbers and hyphens");
        }
        if !ids.insert(&profile.id) {
            bail!("duplicate agent profile id");
        }
        if profile.name.trim().is_empty() || profile.version.trim().is_empty() {
            bail!("agent name and declared version are required");
        }
        if profile
            .node_id
            .as_deref()
            .is_some_and(|id| !node_ids.contains(id))
        {
            bail!("agent references a node absent from the backup");
        }
        for path in [&profile.binary_path, &profile.working_dir]
            .into_iter()
            .chain(profile.config_path.as_ref())
        {
            if !absolute_reference(path) {
                bail!("agent files and working directory must use absolute references");
            }
        }
        if profile.args.len() > 128
            || profile
                .args
                .iter()
                .any(|arg| arg.len() > 8192 || arg.contains('\0'))
        {
            bail!("invalid or oversized agent arguments");
        }
        if crate::redaction::redact_sensitive_args(&profile.args) != profile.args {
            bail!("agent arguments contain secret values");
        }
        if profile.args.iter().any(|arg| arg.contains("{config}")) && profile.config_path.is_none()
        {
            bail!("agent config placeholder has no file reference");
        }
        if profile
            .args
            .iter()
            .any(|arg| arg.contains("{node_id}") || arg.contains("{rpc_url}"))
            && profile.node_id.is_none()
        {
            bail!("agent node placeholder has no association");
        }
        if let Some(endpoint) = &profile.health_url {
            let url = url::Url::parse(endpoint)?;
            let loopback = url.host_str().is_some_and(|host| {
                host == "localhost"
                    || host == "[::1]"
                    || host
                        .parse::<std::net::IpAddr>()
                        .is_ok_and(|ip| ip.is_loopback())
            });
            if !(url.scheme() == "https" || url.scheme() == "http" && loopback)
                || !url.username().is_empty()
                || url.password().is_some()
                || url.query().is_some()
                || url.fragment().is_some()
            {
                bail!("agent health URL must be HTTPS or loopback HTTP without credentials, query or fragment");
            }
        }
    }
    Ok(())
}

fn absolute_reference(path: &Path) -> bool {
    let text = path.to_string_lossy();
    // Backups can move between operating systems; validate reference shape,
    // while the launch path checks existence and the native platform later.
    path.is_absolute()
        || text.starts_with('/')
        || text.starts_with("\\\\")
        || (text.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
            && text.as_bytes().get(1) == Some(&b':')
            && text
                .as_bytes()
                .get(2)
                .is_some_and(|byte| matches!(byte, b'/' | b'\\')))
}
