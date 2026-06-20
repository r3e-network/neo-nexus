use super::RequiredCommand;

pub(super) const WALLET_COMMANDS: &[RequiredCommand] = &[
    RequiredCommand {
        label: "wallet-validate-text",
        fragment: "cargo run -- --validate-wallet",
    },
    RequiredCommand {
        label: "wallet-validate-json",
        fragment: "cargo run -- --validate-wallet-json",
    },
    RequiredCommand {
        label: "wallet-profile-import",
        fragment: "cargo run -- --import-wallet-profile",
    },
];
