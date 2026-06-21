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
