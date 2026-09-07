//! Tests for private network shell command quoting

use crate::private_network::sh_command_tokens;

#[test]
fn sh_command_tokens_single_quotes_the_binary_with_no_arguments() {
    let result = sh_command_tokens("neo-signer", &[]);
    assert_eq!(result, "'neo-signer'");
}

#[test]
fn sh_command_tokens_single_quotes_each_argument_and_joins_with_spaces() {
    let args = vec!["--port".to_string(), "20333".to_string()];
    let result = sh_command_tokens("neo-signer", &args);
    assert_eq!(result, "'neo-signer' '--port' '20333'");
}

#[test]
fn sh_command_tokens_quotes_arguments_containing_spaces() {
    let args = vec!["value with spaces".to_string()];
    let result = sh_command_tokens("command", &args);
    assert_eq!(result, "'command' 'value with spaces'");
}

#[test]
fn sh_command_tokens_escapes_embedded_single_quotes() {
    let args = vec!["it's".to_string()];
    let result = sh_command_tokens("command", &args);
    // A single quote is closed, escaped as \' and reopened.
    assert_eq!(result, "'command' 'it'\\''s'");
}

#[test]
fn sh_command_tokens_preserves_argument_order() {
    let args = vec![
        "first".to_string(),
        "second".to_string(),
        "third".to_string(),
    ];
    let result = sh_command_tokens("cmd", &args);
    assert_eq!(result, "'cmd' 'first' 'second' 'third'");
}
