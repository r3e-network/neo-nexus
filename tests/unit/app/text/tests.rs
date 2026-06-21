use super::*;

#[test]
fn truncate_middle_preserves_both_ends() {
    assert_eq!(
        truncate_middle("/very/long/path/to/file.db", 14),
        "/very...ile.db"
    );
}

#[test]
fn truncate_end_preserves_prefix() {
    assert_eq!(truncate_end("abcdef", 4), "a...");
}

#[test]
fn non_empty_replaces_blank_values() {
    assert_eq!(non_empty("   ", "-"), "-");
    assert_eq!(non_empty("value", "-"), "value");
}

#[test]
fn unix_age_labels_are_human_readable() {
    assert_eq!(format_optional_unix_age(None, 10_000), "never");
    assert_eq!(format_unix_age(10_000, 10_000), "just now");
    assert_eq!(format_unix_age(9_940, 10_000), "1 min ago");
    assert_eq!(format_unix_age(2_800, 10_000), "2 hr ago");
    assert_eq!(format_unix_age(13_600, 10_000), "in 1 hr");
}
