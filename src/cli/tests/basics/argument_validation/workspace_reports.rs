use super::assert_rejects;

#[test]
fn cli_rejects_workspace_report_argument_errors() {
    for args in [
        &["neo-nexus", "--workspace-integrity", "neonexus.db", "extra"][..],
        &["neo-nexus", "--workspace-metrics", "neonexus.db", "extra"][..],
        &[
            "neo-nexus",
            "--workspace-metrics-prometheus",
            "neonexus.db",
            "extra",
        ][..],
        &[
            "neo-nexus",
            "--workspace-integrity-json",
            "neonexus.db",
            "extra",
        ][..],
        &["neo-nexus", "--export-readiness-report"][..],
        &["neo-nexus", "--export-readiness-report", "neonexus.db"][..],
        &["neo-nexus", "--export-support-bundle"][..],
        &["neo-nexus", "--export-support-bundle", "neonexus.db"][..],
        &[
            "neo-nexus",
            "--export-support-bundle",
            "neonexus.db",
            "support",
            "extra",
        ][..],
        &["neo-nexus", "--export-support-bundle-json"][..],
        &["neo-nexus", "--export-support-bundle-json", "neonexus.db"][..],
        &[
            "neo-nexus",
            "--export-support-bundle-json",
            "neonexus.db",
            "support",
            "extra",
        ][..],
        &["neo-nexus", "--export-event-journal"][..],
        &["neo-nexus", "--export-event-journal", "neonexus.db"][..],
        &[
            "neo-nexus",
            "--export-event-journal",
            "neonexus.db",
            "reports",
            "501",
        ][..],
        &[
            "neo-nexus",
            "--export-event-journal",
            "neonexus.db",
            "reports",
            "10",
            "extra",
        ][..],
    ] {
        assert_rejects(args);
    }
}
