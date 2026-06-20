pub(in crate::private_network) const VALIDATION_REPORT_TEXT_FILE: &str = "validation-report.txt";
pub(in crate::private_network) const VALIDATION_REPORT_JSON_FILE: &str = "validation-report.json";
pub(in crate::private_network) const LAUNCH_PACK_SCHEMA_VERSION: u32 = 10;
pub(in crate::private_network) const WALLET_PROVISIONING_SCHEMA_VERSION: u32 = 1;
pub(in crate::private_network) const START_ORDER_FILE: &str = "start-order.txt";
pub(in crate::private_network) const SIGNER_SIDECAR_ROOT: &str = "signers";
pub(in crate::private_network) const WALLET_PROVISIONING_FILE: &str = "wallet-provisioning.json";
pub(in crate::private_network) const WALLET_INSTRUCTIONS_FILE: &str = "wallets/README.md";
pub(in crate::private_network) const WALLET_ROOT: &str = "wallets";
pub(in crate::private_network) const SECRET_PROVISIONING_POLICY: &str =
    "operator-provided-wallets-no-secret-material-in-launch-pack";
pub(in crate::private_network) const COMMITTEE_SECRET_MATERIAL_POLICY: &str =
    "references-only-no-private-keys-or-passwords";
pub(in crate::private_network) const COMMITTEE_PREFLIGHT_POLICY: &str =
    "check-native-wallet-paths-http-endpoints-and-sidecar-commands";
