use super::{ManagerCliOutput, WebLaunch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerMode {
    /// The browser-accessible web workbench — the default experience.
    Web,
    /// A headless manager command printing text/JSON to stdout.
    Cli,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerAction {
    ServeWeb(WebLaunch),
    WriteCli { text: String, exit_code: i32 },
}

impl ManagerAction {
    pub fn mode(&self) -> ManagerMode {
        match self {
            Self::ServeWeb(_) => ManagerMode::Web,
            Self::WriteCli { .. } => ManagerMode::Cli,
        }
    }

    pub fn into_cli_output(self) -> Option<ManagerCliOutput> {
        match self {
            Self::ServeWeb(_) => None,
            Self::WriteCli { text, exit_code } => Some(ManagerCliOutput { text, exit_code }),
        }
    }
}
