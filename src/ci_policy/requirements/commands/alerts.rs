use super::RequiredCommand;

pub(super) const ALERT_COMMANDS: &[RequiredCommand] = &[
    RequiredCommand {
        label: "alert-preview-text",
        fragment: "cargo run -- --alert-preview datadog",
    },
    RequiredCommand {
        label: "alert-preview-json",
        fragment: "cargo run -- --alert-preview-json datadog",
    },
    RequiredCommand {
        label: "alert-preview-text-ready",
        fragment: "alert-preview: ready",
    },
    RequiredCommand {
        label: "alert-preview-json-provider",
        fragment: "\"provider\": \"datadog\"",
    },
    RequiredCommand {
        label: "alert-preview-redacted-header",
        fragment: "DD-API-KEY=<redacted>",
    },
];
