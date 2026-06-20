use std::path::Path;

use super::*;

#[test]
fn parses_quoted_argv_text() {
    let result = parse_argv_text(
        "--config-file \"/Users/me/Neo Config/protocol.yml\" --label 'validator one'",
    );

    assert!(result.is_ok());
    let Ok(args) = result else {
        return;
    };
    assert_eq!(
        args,
        [
            "--config-file",
            "/Users/me/Neo Config/protocol.yml",
            "--label",
            "validator one"
        ]
    );
}

#[test]
fn rejects_unterminated_quoted_argv_text() {
    let result = parse_argv_text("--config \"missing-end");

    assert!(result.is_err_and(|error| error.to_string().contains("unterminated quote")));
}

#[test]
fn formats_argv_with_reversible_quotes() {
    let args = vec![
        "--config-file".to_string(),
        "/Users/me/Neo Config/protocol.yml".to_string(),
        "--label".to_string(),
        "validator \"one\"".to_string(),
        "path\\with\\slashes".to_string(),
        "apostrophe's".to_string(),
        String::new(),
    ];

    let formatted = format_argv(&args);
    assert_eq!(
        formatted,
        "--config-file \"/Users/me/Neo Config/protocol.yml\" --label \"validator \\\"one\\\"\" \"path\\\\with\\\\slashes\" \"apostrophe's\" \"\""
    );

    let reparsed = parse_argv_text(&formatted);
    assert!(reparsed.is_ok());
    let Ok(reparsed) = reparsed else {
        return;
    };
    assert_eq!(reparsed, args);
}

#[test]
fn formats_windows_paths_and_urls_reversibly() {
    let args = vec![
        "--config-file".to_string(),
        r"C:\Users\me\Neo Config\protocol.yml".to_string(),
        "--rpc-url".to_string(),
        "http://127.0.0.1:10332?network=private&height=42".to_string(),
    ];

    let formatted = format_argv(&args);
    let reparsed = parse_argv_text(&formatted);

    assert!(formatted.contains(r#""C:\\Users\\me\\Neo Config\\protocol.yml""#));
    assert!(reparsed.is_ok());
    let Ok(reparsed) = reparsed else {
        return;
    };
    assert_eq!(reparsed, args);
}

#[test]
fn formats_command_with_binary_and_args() {
    let args = vec![
        "node".to_string(),
        "--config-file".to_string(),
        "/Users/me/Neo Config/protocol.yml".to_string(),
    ];

    let command = format_command(Path::new("/Applications/Neo Go/neo-go"), &args);

    assert_eq!(
        command,
        "\"/Applications/Neo Go/neo-go\" node --config-file \"/Users/me/Neo Config/protocol.yml\""
    );
}
