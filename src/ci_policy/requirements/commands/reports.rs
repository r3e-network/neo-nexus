use super::RequiredCommand;

pub(super) const REPORT_COMMANDS: &[RequiredCommand] = &[
    RequiredCommand {
        label: "readiness-report",
        fragment: "cargo run -- --export-readiness-report",
    },
    RequiredCommand {
        label: "support-bundle-text",
        fragment: "cargo run -- --export-support-bundle",
    },
    RequiredCommand {
        label: "support-bundle-json",
        fragment: "cargo run -- --export-support-bundle-json",
    },
    RequiredCommand {
        label: "event-journal",
        fragment: "cargo run -- --export-event-journal",
    },
    RequiredCommand {
        label: "node-config-export-text",
        fragment: "cargo run -- --export-node-configs",
    },
    RequiredCommand {
        label: "node-config-export-json",
        fragment: "cargo run -- --export-node-configs-json",
    },
];
