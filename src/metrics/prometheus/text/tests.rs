use super::{push_header, push_sample};

#[test]
fn prometheus_header_escapes_help_backslashes_and_newlines() {
    let mut output = String::new();
    push_header(
        &mut output,
        "neonexus_test_metric",
        "line one\\two\nline two",
    );

    assert_eq!(
        output,
        "# HELP neonexus_test_metric line one\\\\two\\nline two\n# TYPE neonexus_test_metric gauge\n"
    );
}

#[test]
fn prometheus_sample_escapes_label_values() {
    let mut output = String::new();
    push_sample(
        &mut output,
        "neonexus_test_metric",
        &[("label", "alpha \"one\"\npath\\tail".to_string())],
        1,
    );

    assert_eq!(
        output,
        "neonexus_test_metric{label=\"alpha \\\"one\\\"\\npath\\\\tail\"} 1\n"
    );
}
