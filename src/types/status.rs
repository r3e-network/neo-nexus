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

    pub fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Starting => "Starting",
            Self::Stopped => "Stopped",
            Self::Error => "Error",
        }
    }

    pub fn is_active(self) -> bool {
        matches!(self, Self::Running | Self::Starting)
    }

    pub fn is_running(self) -> bool {
        self == Self::Running
    }

    pub fn is_starting(self) -> bool {
        self == Self::Starting
    }

    pub fn is_stopped(self) -> bool {
        self == Self::Stopped
    }
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

#[cfg(test)]
mod tests {
    use super::NodeStatus;

    #[test]
    fn node_status_operator_labels_are_title_case() {
        assert_eq!(NodeStatus::Running.label(), "Running");
        assert_eq!(NodeStatus::Starting.label(), "Starting");
        assert_eq!(NodeStatus::Stopped.label(), "Stopped");
        assert_eq!(NodeStatus::Error.label(), "Error");
    }

    #[test]
    fn node_status_lifecycle_predicates_capture_operator_safety() {
        assert!(NodeStatus::Running.is_active());
        assert!(NodeStatus::Starting.is_active());
        assert!(!NodeStatus::Stopped.is_active());
        assert!(!NodeStatus::Error.is_active());
        assert!(NodeStatus::Running.is_running());
        assert!(!NodeStatus::Starting.is_running());
        assert!(NodeStatus::Starting.is_starting());
        assert!(!NodeStatus::Running.is_starting());
        assert!(NodeStatus::Stopped.is_stopped());
        assert!(!NodeStatus::Error.is_stopped());
    }
}
