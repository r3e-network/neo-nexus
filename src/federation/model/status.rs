use std::{fmt, str::FromStr};

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteProbeStatus {
    Healthy,
    Degraded,
    Disabled,
    Unreachable,
}

impl RemoteProbeStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Disabled => "disabled",
            Self::Unreachable => "unreachable",
        }
    }
}

impl fmt::Display for RemoteProbeStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

impl FromStr for RemoteProbeStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "healthy" => Ok(Self::Healthy),
            "degraded" => Ok(Self::Degraded),
            "disabled" => Ok(Self::Disabled),
            "unreachable" => Ok(Self::Unreachable),
            other => anyhow::bail!("unsupported remote probe status: {other}"),
        }
    }
}
