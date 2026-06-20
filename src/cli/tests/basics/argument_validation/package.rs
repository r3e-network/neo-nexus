use super::assert_rejects;

#[test]
fn cli_rejects_backup_launch_and_release_argument_errors() {
    for args in [
        &["neo-nexus", "--export-backup"][..],
        &["neo-nexus", "--export-backup", "neonexus.db"][..],
        &["neo-nexus", "--export-backup-json"][..],
        &["neo-nexus", "--export-backup-json", "neonexus.db"][..],
        &["neo-nexus", "--import-backup"][..],
        &["neo-nexus", "--import-backup", "neonexus.db"][..],
        &["neo-nexus", "--import-backup-json"][..],
        &["neo-nexus", "--import-backup-json", "neonexus.db"][..],
        &["neo-nexus", "--validate-backup"][..],
        &["neo-nexus", "--validate-backup-json"][..],
        &["neo-nexus", "--launch-pack-sidecars"][..],
        &["neo-nexus", "--launch-pack-sidecars-json"][..],
        &[
            "neo-nexus",
            "--launch-pack-sidecars",
            "manifest.json",
            "extra",
        ][..],
        &[
            "neo-nexus",
            "--launch-pack-sidecars-json",
            "manifest.json",
            "extra",
        ][..],
        &["neo-nexus", "--package-release"][..],
        &["neo-nexus", "--verify-release-package"][..],
        &["neo-nexus", "--verify-release-package-json"][..],
    ] {
        assert_rejects(args);
    }
}
