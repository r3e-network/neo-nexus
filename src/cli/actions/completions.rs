use anyhow::Result;

use super::suggest::KNOWN_OPTIONS;
use super::{require_arg_count, CliAction};

pub(in crate::cli::actions) fn completions_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--completions")?;
    let script = match args[2].as_str() {
        "bash" => bash_completions(),
        "zsh" => zsh_completions(),
        "fish" => fish_completions(),
        other => {
            anyhow::bail!("unsupported shell for completions: {other}; expected bash, zsh, or fish")
        }
    };
    Ok(CliAction::Print(script))
}

/// The long-form options offered for tab completion. Short aliases (`-V`, `-h`)
/// are omitted because completing two-character flags adds noise.
fn completable_options() -> Vec<&'static str> {
    KNOWN_OPTIONS
        .iter()
        .copied()
        .filter(|option| option.starts_with("--"))
        .collect()
}

fn bash_completions() -> String {
    let options = completable_options().join(" ");
    let mut script = String::new();
    script.push_str("# neo-nexus bash completion\n");
    script.push_str("_neo_nexus() {\n");
    script.push_str(&format!("    local options=\"{options}\"\n"));
    script.push_str(
        "    COMPREPLY=( $(compgen -W \"$options\" -- \"${COMP_WORDS[COMP_CWORD]}\") )\n",
    );
    script.push_str("}\n");
    script.push_str("complete -F _neo_nexus neo-nexus\n");
    script
}

fn zsh_completions() -> String {
    let options = completable_options().join(" ");
    let mut script = String::new();
    script.push_str("#compdef neo-nexus\n");
    script.push_str("# neo-nexus zsh completion\n");
    script.push_str("_neo_nexus() {\n");
    script.push_str(&format!("    local options=({options})\n"));
    script.push_str("    compadd -- $options\n");
    script.push_str("}\n");
    script.push_str("_neo_nexus\n");
    script
}

fn fish_completions() -> String {
    let options = completable_options().join(" ");
    format!("# neo-nexus fish completion\ncomplete -c neo-nexus -f -a \"{options}\"\n")
}

#[cfg(test)]
#[path = "../../../tests/unit/cli/actions/completions/tests.rs"]
mod tests;
