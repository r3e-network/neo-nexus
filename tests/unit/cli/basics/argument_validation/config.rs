use super::assert_rejects;

#[test]
fn cli_rejects_config_argument_errors() {
    for args in [
        &["neo-nexus", "--export-node-configs"][..],
        &["neo-nexus", "--export-node-configs", "neonexus.db"][..],
        &[
            "neo-nexus",
            "--export-node-configs",
            "neonexus.db",
            "configs",
            "extra",
        ][..],
        &["neo-nexus", "--export-node-configs-json"][..],
        &["neo-nexus", "--export-node-configs-json", "neonexus.db"][..],
        &[
            "neo-nexus",
            "--export-node-configs-json",
            "neonexus.db",
            "configs",
            "extra",
        ][..],
        &["neo-nexus", "--generate-node-config"][..],
        &[
            "neo-nexus",
            "--generate-node-config",
            "neo-rs",
            "testnet",
            "rocksdb",
            "10332",
            "10333",
        ][..],
        &[
            "neo-nexus",
            "--generate-node-config-json",
            "neo-rs",
            "testnet",
            "rocksdb",
            "10332",
            "10333",
        ][..],
        &[
            "neo-nexus",
            "--generate-node-config",
            "neo-rs",
            "testnet",
            "leveldb",
            "10332",
            "10333",
            "config.toml",
        ][..],
        &["neo-nexus", "--validate-node-config"][..],
        &[
            "neo-nexus",
            "--validate-node-config",
            "neo-rs",
            "testnet",
            "rocksdb",
            "10332",
            "10333",
        ][..],
        &[
            "neo-nexus",
            "--validate-node-config-json",
            "neo-rs",
            "testnet",
            "rocksdb",
            "10332",
            "10333",
        ][..],
        &[
            "neo-nexus",
            "--validate-node-config",
            "neo-rs",
            "testnet",
            "leveldb",
            "10332",
            "10333",
            "config.toml",
        ][..],
    ] {
        assert_rejects(args);
    }
}
