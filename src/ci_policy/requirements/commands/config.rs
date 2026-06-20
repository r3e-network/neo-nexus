use super::RequiredCommand;

pub(super) const CONFIG_COMMANDS: &[RequiredCommand] = &[
    RequiredCommand {
        label: "neo-rs-config-generate-text",
        fragment: "cargo run -- --generate-node-config neo-rs",
    },
    RequiredCommand {
        label: "neo-rs-config-generate-json",
        fragment: "cargo run -- --generate-node-config-json neo-rs",
    },
    RequiredCommand {
        label: "neo-rs-config-validate-text",
        fragment: "cargo run -- --validate-node-config neo-rs",
    },
    RequiredCommand {
        label: "neo-rs-config-validate-json",
        fragment: "cargo run -- --validate-node-config-json neo-rs",
    },
];
