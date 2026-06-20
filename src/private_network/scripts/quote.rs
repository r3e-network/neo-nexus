pub(in crate::private_network) fn sh_command_tokens(
    binary_path: &str,
    arguments: &[String],
) -> String {
    std::iter::once(sh_literal(binary_path))
        .chain(arguments.iter().map(|argument| sh_literal(argument)))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn sh_arg_list_suffix(arguments: &[String]) -> String {
    if arguments.is_empty() {
        String::new()
    } else {
        format!(
            " {}",
            arguments
                .iter()
                .map(|argument| sh_literal(argument))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

pub(super) fn sh_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(super) fn ps_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| ps_literal(value))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn ps_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
