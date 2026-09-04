use super::{line_drift, ConfigLineDrift};

#[test]
fn identical_texts_have_no_drift() {
    let text = "alpha = 1\nbeta = 2\n";
    assert_eq!(
        line_drift(text, text),
        ConfigLineDrift {
            unexpected_lines: 0,
            missing_lines: 0,
            unexpected_samples: Vec::new(),
        }
    );
}

/// A trailing newline is formatting, not content — `lines()` ignores it.
#[test]
fn a_trailing_newline_is_not_drift() {
    assert!(line_drift("alpha = 1\n", "alpha = 1").is_empty());
    assert!(line_drift("alpha = 1", "alpha = 1\n").is_empty());
}

#[test]
fn a_hand_edited_line_is_unexpected_and_quoted() {
    let drift = line_drift(
        "alpha = 1\nbeta = 2\n",
        "alpha = 1\nbeta = 99\nlegacy = true\n",
    );
    assert_eq!(drift.unexpected_lines, 2);
    assert_eq!(drift.missing_lines, 1);
    assert_eq!(drift.unexpected_samples, vec!["beta = 99", "legacy = true"]);
}

#[test]
fn a_line_only_in_the_render_is_missing_not_unexpected() {
    let drift = line_drift("alpha = 1\nbeta = 2\n", "alpha = 1\n");
    assert_eq!(drift.unexpected_lines, 0);
    assert_eq!(drift.missing_lines, 1);
    assert!(drift.unexpected_samples.is_empty());
}

/// Multiset semantics: an operator duplicating a block counts each extra
/// occurrence, and the sample quotes the line once.
#[test]
fn a_duplicated_line_counts_each_extra_occurrence() {
    let drift = line_drift("seed = a\n", "seed = a\nseed = a\nseed = a\n");
    assert_eq!(drift.unexpected_lines, 2);
    assert_eq!(drift.missing_lines, 0);
    assert_eq!(drift.unexpected_samples, vec!["seed = a"]);
}

#[test]
fn samples_stop_at_three_and_long_lines_are_truncated() {
    let long_line = format!("{}x", "x".repeat(200));
    let disk = format!("one\ntwo\n{long_line}\nthree\nfour\n");
    let drift = line_drift("", &disk);
    assert_eq!(drift.unexpected_lines, 5);
    assert_eq!(drift.unexpected_samples.len(), 3);
    assert_eq!(drift.unexpected_samples[2].chars().count(), 81);
    assert!(drift.unexpected_samples[2].ends_with('…'));
}
