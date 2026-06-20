use super::RequiredCommand;

pub(super) const BACKUP_COMMANDS: &[RequiredCommand] = &[
    RequiredCommand {
        label: "backup-export-text",
        fragment: "cargo run -- --export-backup",
    },
    RequiredCommand {
        label: "backup-export-json",
        fragment: "cargo run -- --export-backup-json",
    },
    RequiredCommand {
        label: "backup-validate-text",
        fragment: "cargo run -- --validate-backup",
    },
    RequiredCommand {
        label: "backup-validate-json",
        fragment: "cargo run -- --validate-backup-json",
    },
    RequiredCommand {
        label: "backup-import-text",
        fragment: "cargo run -- --import-backup",
    },
    RequiredCommand {
        label: "backup-import-json",
        fragment: "cargo run -- --import-backup-json",
    },
];
