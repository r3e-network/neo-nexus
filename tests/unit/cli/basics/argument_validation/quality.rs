use super::assert_rejects;

#[test]
fn cli_rejects_unknown_singleton_or_quality_argument_errors() {
    for args in [
        &["neo-nexus", "--bogus"][..],
        &["neo-nexus", "--version", "--self-check"][..],
        &["neo-nexus", "--runtime-smoke", "neo-go"][..],
        &["neo-nexus", "--runtime-smoke-json", "neo-go"][..],
        &["neo-nexus", "--rpc-health"][..],
        &["neo-nexus", "--rpc-health-json"][..],
        &["neo-nexus", "--rpc-health-json", "127.0.0.1:1", "extra"][..],
        &["neo-nexus", "--workspace-readiness"][..],
        &["neo-nexus", "--workspace-readiness-json"][..],
        &["neo-nexus", "--workspace-metrics"][..],
        &["neo-nexus", "--workspace-metrics-json"][..],
        &["neo-nexus", "--workspace-metrics-prometheus"][..],
        &["neo-nexus", "--workspace-integrity"][..],
        &["neo-nexus", "--workspace-integrity-json"][..],
        &["neo-nexus", "--source-purity"][..],
        &["neo-nexus", "--source-purity-json"][..],
        &["neo-nexus", "--source-quality"][..],
        &["neo-nexus", "--source-quality-json"][..],
        &["neo-nexus", "--native-ui-audit"][..],
        &["neo-nexus", "--native-ui-audit-json"][..],
        &["neo-nexus", "--ci-policy"][..],
        &["neo-nexus", "--ci-policy-json"][..],
        &["neo-nexus", "--source-purity", ".", "extra"][..],
        &["neo-nexus", "--source-purity-json", ".", "extra"][..],
        &["neo-nexus", "--source-quality", ".", "extra"][..],
        &["neo-nexus", "--source-quality-json", ".", "extra"][..],
        &["neo-nexus", "--native-ui-audit", ".", "extra"][..],
        &["neo-nexus", "--native-ui-audit-json", ".", "extra"][..],
        &["neo-nexus", "--ci-policy", ".", "extra"][..],
        &["neo-nexus", "--ci-policy-json", ".", "extra"][..],
    ] {
        assert_rejects(args);
    }
}
