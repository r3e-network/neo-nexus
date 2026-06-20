use super::RequiredCommand;

pub(super) const QUALITY_COMMANDS: &[RequiredCommand] = &[
    RequiredCommand {
        label: "rustfmt",
        fragment: "cargo fmt --all --check",
    },
    RequiredCommand {
        label: "cargo-check",
        fragment: "cargo check",
    },
    RequiredCommand {
        label: "clippy-deny-warnings",
        fragment: "cargo clippy --all-targets -- -D warnings",
    },
    RequiredCommand {
        label: "cargo-test-lib",
        fragment: "cargo test --lib",
    },
    RequiredCommand {
        label: "cargo-test-ci-policy",
        fragment: "cargo test --test ci_policy",
    },
    RequiredCommand {
        label: "cargo-test-domain",
        fragment: "cargo test --test domain",
    },
    RequiredCommand {
        label: "cargo-test-repository",
        fragment: "cargo test --test repository",
    },
    RequiredCommand {
        label: "debug-self-check",
        fragment: "cargo run -- --self-check",
    },
    RequiredCommand {
        label: "ci-policy-text",
        fragment: "cargo run -- --ci-policy .github/workflows/ci.yml",
    },
    RequiredCommand {
        label: "ci-policy-json",
        fragment: "cargo run -- --ci-policy-json .github/workflows/ci.yml",
    },
    RequiredCommand {
        label: "source-purity-text",
        fragment: "cargo run -- --source-purity .",
    },
    RequiredCommand {
        label: "source-purity-json",
        fragment: "cargo run -- --source-purity-json .",
    },
    RequiredCommand {
        label: "source-quality-src-text",
        fragment: "cargo run -- --source-quality src",
    },
    RequiredCommand {
        label: "source-quality-src-json",
        fragment: "cargo run -- --source-quality-json src",
    },
    RequiredCommand {
        label: "source-quality-tests-text",
        fragment: "cargo run -- --source-quality tests",
    },
    RequiredCommand {
        label: "source-quality-tests-json",
        fragment: "cargo run -- --source-quality-json tests",
    },
    RequiredCommand {
        label: "source-quality-root-text",
        fragment: "cargo run -- --source-quality .",
    },
    RequiredCommand {
        label: "source-quality-root-json",
        fragment: "cargo run -- --source-quality-json .",
    },
    RequiredCommand {
        label: "native-ui-audit-text",
        fragment: "cargo run -- --native-ui-audit .",
    },
    RequiredCommand {
        label: "native-ui-audit-json",
        fragment: "cargo run -- --native-ui-audit-json .",
    },
];
