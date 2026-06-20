use std::{
    fmt,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use crate::{launch::LaunchPlan, types::NodeConfig};

pub(super) const DEFAULT_STOP_GRACE_PERIOD: Duration = Duration::from_secs(5);

pub struct ProcessStart {
    pub pid: u32,
    pub log_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStop {
    pub process_id: String,
    pub pid: u32,
    pub log_path: PathBuf,
    pub graceful: bool,
    pub forced: bool,
    pub exit_code: Option<i32>,
}

impl ProcessStop {
    pub fn operator_summary(&self) -> String {
        let mode = if self.forced {
            "forced"
        } else if self.graceful {
            "graceful"
        } else {
            "stopped"
        };
        match self.exit_code {
            Some(code) => format!("pid {} {mode}, exit {code}", self.pid),
            None => format!("pid {} {mode}", self.pid),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessExit {
    pub process_id: String,
    pub node_id: String,
    pub pid: u32,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManagedProcessKind {
    Node,
    Sidecar,
    Helper,
}

impl ManagedProcessKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::Sidecar => "sidecar",
            Self::Helper => "helper",
        }
    }
}

impl fmt::Display for ManagedProcessKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManagedProcessSpec {
    pub id: String,
    pub kind: ManagedProcessKind,
    pub label: String,
    pub binary_path: PathBuf,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub display_command: String,
}

impl ManagedProcessSpec {
    pub fn for_node(node: &NodeConfig, plan: &LaunchPlan) -> Self {
        Self {
            id: node.id.clone(),
            kind: ManagedProcessKind::Node,
            label: node.name.clone(),
            binary_path: plan.binary_path.clone(),
            args: plan.args.clone(),
            working_dir: plan.working_dir.clone(),
            display_command: plan.display_command.clone(),
        }
    }
}

pub(super) fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
