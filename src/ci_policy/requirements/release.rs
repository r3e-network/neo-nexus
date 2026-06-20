use super::RequiredCommand;

pub(in crate::ci_policy) const REQUIRED_RELEASE_COMMANDS: [RequiredCommand; 6] = [
    RequiredCommand {
        label: "release-build",
        fragment: "cargo build --release",
    },
    RequiredCommand {
        label: "release-self-check-unix",
        fragment: "./target/release/neo-nexus --self-check",
    },
    RequiredCommand {
        label: "release-self-check-windows",
        fragment: ".\\target\\release\\neo-nexus.exe --self-check",
    },
    RequiredCommand {
        label: "release-package",
        fragment: "--package-release dist",
    },
    RequiredCommand {
        label: "release-verify-text",
        fragment: "--verify-release-package dist",
    },
    RequiredCommand {
        label: "release-verify-json",
        fragment: "--verify-release-package-json dist",
    },
];
