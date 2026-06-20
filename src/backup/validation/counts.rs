#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(in crate::backup) struct BackupValidationCounts {
    pub(in crate::backup) plugin_state_count: usize,
    pub(in crate::backup) plugin_installation_count: usize,
}
