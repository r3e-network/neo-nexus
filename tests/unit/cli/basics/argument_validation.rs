use super::super::*;

mod config;
mod package;
mod quality;
mod workspace_reports;

fn assert_rejects(args: &[&str]) {
    assert!(
        action_from_args(args.iter().copied()).is_err(),
        "expected CLI rejection for {args:?}"
    );
}

#[test]
fn cli_distinguishes_missing_from_extra_arguments() {
    let missing = action_from_args(["neo-nexus", "--rpc-health"])
        .expect_err("missing port must be rejected")
        .to_string();
    assert!(
        missing.contains("missing required arguments"),
        "missing-argument error should say so, got: {missing}"
    );
    assert!(
        !missing.contains("does not accept extra"),
        "missing-argument error must not claim extra arguments, got: {missing}"
    );

    let extra = action_from_args(["neo-nexus", "--source-quality", "src", "extra"])
        .expect_err("extra argument must be rejected")
        .to_string();
    assert!(
        extra.contains("does not accept extra arguments"),
        "extra-argument error should say so, got: {extra}"
    );
}

#[test]
fn cli_suggests_the_closest_option_for_a_typo() {
    let error = action_from_args(["neo-nexus", "--rpc-helth", "8080"])
        .expect_err("a mistyped option must be rejected")
        .to_string();
    assert!(
        error.contains("did you mean --rpc-health"),
        "expected a suggestion for the typo, got: {error}"
    );
}
