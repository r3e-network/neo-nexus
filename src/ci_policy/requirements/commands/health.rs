use super::RequiredCommand;

pub(super) const HEALTH_COMMANDS: &[RequiredCommand] = &[
    RequiredCommand {
        label: "neo-rs-runtime-smoke-text",
        fragment: "cargo run -- --runtime-smoke neo-rs",
    },
    RequiredCommand {
        label: "neo-rs-runtime-smoke-json",
        fragment: "cargo run -- --runtime-smoke-json neo-rs",
    },
    RequiredCommand {
        label: "neo-rs-runtime-smoke-text-passed",
        fragment: "runtime-smoke: passed",
    },
    RequiredCommand {
        label: "neo-rs-runtime-smoke-text-sha256",
        fragment: "runtime-binary-sha256:",
    },
    RequiredCommand {
        label: "neo-rs-runtime-smoke-json-passed",
        fragment: "\"status\": \"passed\"",
    },
    RequiredCommand {
        label: "neo-rs-runtime-smoke-json-binary-evidence",
        fragment: "\"binary_evidence\"",
    },
    RequiredCommand {
        label: "neo-rs-runtime-smoke-json-verified-binary",
        fragment: "\"status\": \"verified\"",
    },
    RequiredCommand {
        label: "neo-rs-runtime-smoke-json-sha256",
        fragment: "\"sha256\":",
    },
    RequiredCommand {
        label: "rpc-health-text",
        fragment: "cargo run -- --rpc-health 127.0.0.1:1",
    },
    RequiredCommand {
        label: "rpc-health-json",
        fragment: "cargo run -- --rpc-health-json 127.0.0.1:1",
    },
    RequiredCommand {
        label: "workspace-readiness",
        fragment: "cargo run -- --workspace-readiness",
    },
    RequiredCommand {
        label: "workspace-metrics-text",
        fragment: "cargo run -- --workspace-metrics",
    },
    RequiredCommand {
        label: "workspace-metrics-json",
        fragment: "cargo run -- --workspace-metrics-json",
    },
    RequiredCommand {
        label: "workspace-metrics-prometheus",
        fragment: "cargo run -- --workspace-metrics-prometheus",
    },
    RequiredCommand {
        label: "workspace-integrity",
        fragment: "cargo run -- --workspace-integrity",
    },
];
