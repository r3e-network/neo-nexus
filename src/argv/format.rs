use std::path::Path;

pub fn format_argv(args: &[String]) -> String {
    args.iter()
        .map(|arg| format_argv_token(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_command(binary_path: &Path, args: &[String]) -> String {
    std::iter::once(format_argv_token(&binary_path.display().to_string()))
        .chain(args.iter().map(|arg| format_argv_token(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_argv_token(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }

    if value.chars().all(is_unquoted_display_character) {
        return value.to_string();
    }

    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn is_unquoted_display_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || "/._:-=".contains(character)
}
