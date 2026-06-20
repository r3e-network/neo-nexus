use std::{fmt, str::FromStr};

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Stopped,
    Starting,
    Running,
    Error,
}

impl NodeStatus {
    pub const ALL: [Self; 4] = [Self::Running, Self::Starting, Self::Stopped, Self::Error];
}

impl fmt::Display for NodeStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Error => "error",
        })
    }
}

impl FromStr for NodeStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "stopped" => Ok(Self::Stopped),
            "starting" => Ok(Self::Starting),
            "running" => Ok(Self::Running),
            "error" => Ok(Self::Error),
            other => anyhow::bail!("unsupported node status: {other}"),
        }
    }
}
