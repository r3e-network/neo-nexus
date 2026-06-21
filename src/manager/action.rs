use super::ManagerCliOutput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerMode {
    Gui,
    Cli,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerAction {
    LaunchGui,
    WriteCli { text: String, exit_code: i32 },
}

impl ManagerAction {
    pub fn mode(&self) -> ManagerMode {
        match self {
            Self::LaunchGui => ManagerMode::Gui,
            Self::WriteCli { .. } => ManagerMode::Cli,
        }
    }

    pub fn is_gui(&self) -> bool {
        self.mode() == ManagerMode::Gui
    }

    pub fn into_cli_output(self) -> Option<ManagerCliOutput> {
        match self {
            Self::LaunchGui => None,
            Self::WriteCli { text, exit_code } => Some(ManagerCliOutput { text, exit_code }),
        }
    }
}
