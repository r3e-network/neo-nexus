use super::*;

#[test]
fn bash_completion_lists_every_long_option() {
    let script = bash_completions();
    assert!(script.contains("complete -F _neo_nexus neo-nexus"));
    assert!(script.contains("compgen -W"));
    for option in completable_options() {
        assert!(script.contains(option), "bash script is missing {option}");
    }
}

#[test]
fn each_shell_script_carries_its_signature() {
    assert!(zsh_completions().contains("#compdef neo-nexus"));
    assert!(zsh_completions().contains("compadd -- $options"));
    assert!(fish_completions().contains("complete -c neo-nexus -f -a"));
}

#[test]
fn completions_only_offer_long_form_flags() {
    let options = completable_options();
    assert!(options.iter().all(|option| option.starts_with("--")));
    assert!(options.contains(&"--completions"));
    assert!(!options.contains(&"-V"));
}

#[test]
fn completions_dispatch_rejects_missing_or_unknown_shell() {
    assert!(crate::cli::action_from_args(["neo-nexus", "--completions"]).is_err());
    let unknown = crate::cli::action_from_args(["neo-nexus", "--completions", "tcsh"])
        .expect_err("an unknown shell must be rejected")
        .to_string();
    assert!(unknown.contains("unsupported shell for completions"));
}

#[test]
fn completions_dispatch_emits_a_script() {
    let action = crate::cli::action_from_args(["neo-nexus", "--completions", "fish"])
        .expect("fish completions should dispatch");
    assert!(
        matches!(action, crate::cli::CliAction::Print(script) if script.contains("complete -c neo-nexus")),
        "expected a printed fish completion script"
    );
}
