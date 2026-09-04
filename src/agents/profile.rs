use crate::{
    repository::Repository,
    supervisor::{ManagedProcessKind, ManagedProcessSpec},
    types::*,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentKind {
    Hermes,
    Signer,
    Plugin,
    Sidecar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Stopped,
    Starting,
    Running,
    Crashed,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub kind: AgentKind,
    pub node_id: Option<String>,
    /// Operator-declared release, pinned to the actual binary hash on save.
    pub version: String,
    pub binary_path: PathBuf,
    pub working_dir: PathBuf,
    pub args: Vec<String>,
    pub config_path: Option<PathBuf>,
    pub health_url: Option<String>,
    pub auto_restart: bool,
    #[serde(default)]
    pub binary_sha256: String,
    #[serde(default)]
    pub config_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRecord {
    pub profile: AgentProfile,
    pub status: AgentStatus,
    pub pid: Option<u32>,
    #[serde(default)]
    pub process_started_at: Option<u64>,
    pub desired_running: bool,
    pub restart_attempts: u32,
    pub restart_after: Option<u64>,
    pub healthy: Option<bool>,
    pub last_health_at: u64,
}

impl AgentProfile {
    pub fn process_id(&self) -> String {
        format!("agent:{}", self.id)
    }

    pub(super) fn validate(&self, repository: &Repository) -> Result<()> {
        if self.id.is_empty()
            || !self
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        {
            bail!("agent id must contain only letters, numbers and hyphens");
        }
        if self.name.trim().is_empty() || self.version.trim().is_empty() {
            bail!("name and declared version are required");
        }
        if !self.binary_path.is_absolute() || !self.binary_path.is_file() {
            bail!("agent executable must be an existing absolute file path");
        }
        if self
            .binary_path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("bat") || s.eq_ignore_ascii_case("cmd"))
        {
            bail!("use a native executable, not a Windows shell batch file");
        }
        if !self.working_dir.is_absolute() || !self.working_dir.is_dir() {
            bail!("working directory must be an existing absolute directory");
        }
        if let Some(path) = &self.config_path {
            if !path.is_absolute() || !path.is_file() {
                bail!("configuration must reference an existing absolute file path");
            }
        }
        if self.args.len() > 128
            || self
                .args
                .iter()
                .any(|arg| arg.len() > 8192 || arg.contains('\0'))
        {
            bail!("invalid or oversized arguments");
        }
        if crate::redaction::redact_sensitive_args(&self.args) != self.args {
            bail!("arguments cannot contain secret values; supply credentials through the agent's configuration file");
        }
        if let Some(id) = &self.node_id {
            if !repository.list_nodes()?.iter().any(|node| node.id == *id) {
                bail!("associated node no longer exists");
            }
        }
        if let Some(endpoint) = &self.health_url {
            let url = url::Url::parse(endpoint).context("invalid health URL")?;
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
                bail!("health URL requires HTTPS or loopback HTTP, without credentials, query or fragment");
            }
        }
        self.spec(repository)?;
        if self.kind == AgentKind::Hermes {
            super::hermes::validate(self)?;
        }
        Ok(())
    }

    pub(super) fn spec(&self, repository: &Repository) -> Result<ManagedProcessSpec> {
        let node = self
            .node_id
            .as_ref()
            .map(|id| {
                repository
                    .list_nodes()
                    .map(|nodes| nodes.into_iter().find(|node| &node.id == id))
            })
            .transpose()?
            .flatten();
        let mut args = Vec::new();
        for template in &self.args {
            let mut arg = template.clone();
            if arg.contains("{config}") {
                let config = self
                    .config_path
                    .as_ref()
                    .context("{config} requires a configuration file")?;
                arg = arg.replace("{config}", &config.to_string_lossy());
            }
            if arg.contains("{node_id}") || arg.contains("{rpc_url}") {
                let node = node
                    .as_ref()
                    .context("node placeholders require an associated node")?;
                arg = arg
                    .replace("{node_id}", &node.id)
                    .replace("{rpc_url}", &format!("http://127.0.0.1:{}", node.rpc_port));
            }
            args.push(arg);
        }
        if self.kind == AgentKind::Hermes {
            args = super::hermes::arguments();
        }
        Ok(ManagedProcessSpec {
            id: self.process_id(),
            kind: ManagedProcessKind::Sidecar,
            label: self.name.clone(),
            binary_path: self.binary_path.clone(),
            args,
            working_dir: self.working_dir.clone(),
            display_command: format!("agent {} ({:?}, {})", self.name, self.kind, self.version),
        })
    }

    /// Adapt to the existing executable-identity-aware PID termination API.
    pub(super) fn identity(&self, pid: Option<u32>) -> NodeConfig {
        NodeConfig {
            id: self.process_id(),
            name: self.name.clone(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: self.binary_path.clone(),
            args: if self.kind == AgentKind::Hermes {
                super::hermes::arguments()
            } else {
                vec![]
            },
            runtime_version: self.version.clone(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 0,
            p2p_port: 0,
            ws_port: None,
            status: NodeStatus::Running,
            pid,
        }
    }
}

pub(super) fn digest(path: &Path) -> Result<String> {
    let mut file = File::open(path).context("cannot open referenced agent file")?;
    let mut hash = Sha256::new();
    let mut bytes = [0u8; 65536];
    loop {
        let read = file.read(&mut bytes)?;
        if read == 0 {
            break;
        }
        hash.update(&bytes[..read]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub(super) fn code_digest(profile: &AgentProfile) -> Result<String> {
    let executable = digest(&profile.binary_path)?;
    if profile.kind != AgentKind::Hermes {
        return Ok(executable);
    }
    // The Python executable alone cannot identify an editable Hermes release.
    // Hash the installed Python source as well; virtualenvs/data/secrets are excluded.
    let mut files = Vec::new();
    for entry in std::fs::read_dir(&profile.working_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && (entry.path().extension().is_some_and(|ext| ext == "py")
                || entry.file_name() == "pyproject.toml")
        {
            files.push(entry.path());
        }
    }
    for directory in [
        "agent",
        "gateway",
        "hermes_cli",
        "tools",
        "cron",
        "providers",
        "tui_gateway",
        "acp_adapter",
    ] {
        let path = profile.working_dir.join(directory);
        if path.is_dir() {
            collect_python(&path, &mut files)?;
        }
    }
    files.sort();
    let mut hash = Sha256::new();
    hash.update(executable);
    for file in files {
        hash.update(
            file.strip_prefix(&profile.working_dir)?
                .to_string_lossy()
                .as_bytes(),
        );
        hash.update([0]);
        hash.update(digest(&file)?);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn collect_python(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() && entry.file_name() != "__pycache__" {
            collect_python(&entry.path(), files)?;
        } else if kind.is_file() && entry.path().extension().is_some_and(|ext| ext == "py") {
            files.push(entry.path());
        }
    }
    Ok(())
}

pub(super) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |time| time.as_secs())
}

pub(super) fn process_started_at(pid: u32) -> Option<u64> {
    let pid = sysinfo::Pid::from_u32(pid);
    let system = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing().with_processes(sysinfo::ProcessRefreshKind::nothing()),
    );
    system.process(pid).map(|process| process.start_time())
}
