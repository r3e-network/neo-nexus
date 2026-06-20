use crate::*;

#[test]
fn log_reader_returns_empty_snapshot_for_missing_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("missing.log");

    let snapshot = LogReader::snapshot(&path, 64).unwrap();

    assert_eq!(snapshot.path, path);
    assert!(!snapshot.exists);
    assert_eq!(snapshot.bytes, 0);
    assert!(snapshot.lines.is_empty());
    assert!(!snapshot.truncated);
}

#[test]
fn log_reader_keeps_tail_window_without_partial_first_line() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("node.log");
    std::fs::write(&path, "line-01\nline-02\nline-03\nline-04\n").unwrap();

    let snapshot = LogReader::snapshot(&path, 17).unwrap();

    assert!(snapshot.exists);
    assert!(snapshot.truncated);
    assert_eq!(snapshot.lines, ["line-03", "line-04"]);
}

#[test]
fn log_reader_clear_truncates_or_creates_log_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("nested").join("node.log");
    LogReader::clear(&path).unwrap();
    std::fs::write(&path, "old output").unwrap();

    LogReader::clear(&path).unwrap();

    let snapshot = LogReader::snapshot(&path, 64).unwrap();
    assert!(snapshot.exists);
    assert_eq!(snapshot.bytes, 0);
    assert!(snapshot.lines.is_empty());
}

#[test]
fn log_reader_filters_lines_case_insensitively_and_keeps_line_numbers() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("node.log");
    std::fs::write(
        &path,
        "INFO node started\nwarning peer timeout\nERROR storage failure\n",
    )
    .unwrap();
    let snapshot = LogReader::snapshot(&path, 512).unwrap();

    let matches = LogReader::filtered_lines(&snapshot, "error");
    let all = LogReader::filtered_lines(&snapshot, "  ");

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].number, 3);
    assert_eq!(matches[0].text, "ERROR storage failure");
    assert_eq!(all.len(), 3);
}
