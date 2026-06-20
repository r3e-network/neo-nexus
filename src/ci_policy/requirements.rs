mod commands;
mod forbidden;
mod release;

pub(super) use commands::required_commands;
pub(super) use forbidden::FORBIDDEN_CI_MARKERS;
pub(super) use release::REQUIRED_RELEASE_COMMANDS;

#[derive(Debug, Clone, Copy)]
pub(super) struct RequiredCommand {
    pub(super) label: &'static str,
    pub(super) fragment: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ForbiddenCiMarker {
    pub(super) marker: &'static str,
    pub(super) message: &'static str,
}

pub(super) const REQUIRED_OS: [&str; 3] = ["ubuntu-latest", "macos-latest", "windows-latest"];
