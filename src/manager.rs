use anyhow::Result;

use crate::cli::{self, CliAction};

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
}

pub fn action_from_args<I, S>(args: I) -> Result<ManagerAction>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    Ok(match cli::action_from_args(args)? {
        CliAction::RunGui => ManagerAction::LaunchGui,
        CliAction::Print(text) => ManagerAction::WriteCli { text, exit_code: 0 },
        CliAction::PrintWithExitCode { text, exit_code } => {
            ManagerAction::WriteCli { text, exit_code }
        }
    })
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::{action_from_args, ManagerAction, ManagerMode};

    #[test]
    fn manager_classifies_default_gui_and_explicit_cli_modes() -> anyhow::Result<()> {
        let gui = action_from_args(["neo-nexus"])?;
        assert_eq!(gui, ManagerAction::LaunchGui);
        assert_eq!(gui.mode(), ManagerMode::Gui);

        let help = action_from_args(["neo-nexus", "--help"])?;
        assert_eq!(help.mode(), ManagerMode::Cli);
        assert!(matches!(
            help,
            ManagerAction::WriteCli {
                text,
                exit_code: 0,
            } if text.contains("NeoNexus")
        ));
        Ok(())
    }

    #[test]
    fn manager_preserves_cli_exit_code_without_gui_dependencies() -> anyhow::Result<()> {
        let root = std::env::temp_dir().join(format!(
            "neo-nexus-manager-source-purity-{}",
            std::process::id()
        ));
        if root.exists() {
            std::fs::remove_dir_all(&root)?;
        }
        std::fs::create_dir_all(&root)?;
        std::fs::write(root.join("package.json"), "{}")?;

        let action = action_from_args(vec![
            "neo-nexus".to_string(),
            "--source-purity".to_string(),
            root.to_str()
                .context("temporary source purity path is not valid UTF-8")?
                .to_string(),
        ])?;
        std::fs::remove_dir_all(&root)?;

        assert_eq!(action.mode(), ManagerMode::Cli);
        assert!(matches!(
            action,
            ManagerAction::WriteCli {
                text,
                exit_code: 1,
            } if text.contains("source-purity: failed")
        ));
        Ok(())
    }
}
