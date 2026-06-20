use super::{rules::marker_token, SourceQualityChecker, MAX_RUST_SOURCE_LINES};

#[test]
fn source_quality_accepts_clean_rust_source_tree() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    std::fs::write(
        temp_dir.path().join("src").join("lib.rs"),
        "pub fn checked(value: Option<u8>) -> bool { value.is_some() }\n",
    )?;

    let report = SourceQualityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(report.is_success(), "{}", report.to_cli_text());
    assert_eq!(report.status, "ok");
    assert_eq!(report.scanned_files, 1);
    assert_eq!(report.finding_count, 0);
    Ok(())
}

#[test]
fn source_quality_rejects_panic_oriented_markers() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    let marker = marker_token("unwrap", '(');
    std::fs::write(
        temp_dir.path().join("src").join("lib.rs"),
        format!("pub fn unchecked(value: Option<u8>) -> u8 {{ value.{marker}) }}\n"),
    )?;
    std::fs::create_dir_all(temp_dir.path().join("target"))?;
    let skipped_marker = marker_token("panic", '!');
    std::fs::write(
        temp_dir.path().join("target").join("generated.rs"),
        format!("pub fn generated() {{ {skipped_marker}(\"skip\"); }}\n"),
    )?;

    let report = SourceQualityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(!report.is_success());
    assert_eq!(report.status, "failed");
    assert_eq!(report.finding_count, 1);
    assert_eq!(report.findings[0].path, "src/lib.rs");
    assert_eq!(report.findings[0].marker, marker);
    assert!(report
        .skipped_directories
        .iter()
        .any(|path| path == "target"));
    Ok(())
}

#[test]
fn source_quality_rejects_document_style_native_layout_markers() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("src").join("app"))?;
    let scroll_marker = marker_token("ScrollArea", ':');
    let table_marker = marker_token("TableBuilder", ':');
    std::fs::write(
        temp_dir.path().join("src").join("app").join("view.rs"),
        format!(
            "fn render(ui: &mut egui::Ui) {{ egui::{scroll_marker}:vertical().show(ui, |_| {{}}); let _ = egui_extras::{table_marker}:new(ui); }}\n"
        ),
    )?;

    let report = SourceQualityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(!report.is_success());
    assert_eq!(report.status, "failed");
    assert_eq!(report.finding_count, 2);
    assert!(report.findings.iter().any(|finding| {
        finding.marker == scroll_marker && finding.category == "document-style-native-layout"
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.marker == table_marker && finding.category == "document-style-native-layout"
    }));
    Ok(())
}

#[test]
fn source_quality_rejects_oversized_rust_source_files() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    let oversized = (0..=MAX_RUST_SOURCE_LINES)
        .map(|index| format!("pub const VALUE_{index}: usize = {index};"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(temp_dir.path().join("src").join("giant.rs"), oversized)?;

    let report = SourceQualityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(!report.is_success());
    assert_eq!(report.status, "failed");
    assert_eq!(report.finding_count, 1);
    assert_eq!(report.findings[0].path, "src/giant.rs");
    assert_eq!(report.findings[0].category, "oversized-rust-file");
    assert_eq!(
        report.findings[0].marker,
        format!(
            "{} lines > {MAX_RUST_SOURCE_LINES}",
            MAX_RUST_SOURCE_LINES + 1
        )
    );
    Ok(())
}

#[test]
fn source_quality_rejects_files_above_professional_module_budget() -> anyhow::Result<()> {
    assert_eq!(MAX_RUST_SOURCE_LINES, 200);
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    let oversized = (0..=MAX_RUST_SOURCE_LINES)
        .map(|index| format!("pub const PROFESSIONAL_VALUE_{index}: usize = {index};"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(temp_dir.path().join("src").join("near_limit.rs"), oversized)?;

    let report = SourceQualityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(!report.is_success());
    assert_eq!(report.finding_count, 1);
    assert_eq!(report.findings[0].path, "src/near_limit.rs");
    assert_eq!(report.findings[0].category, "oversized-rust-file");
    Ok(())
}

#[test]
fn source_quality_allows_test_assertion_shortcuts_but_keeps_budget() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("tests"))?;
    let assertion_marker = marker_token("unwrap", '(');
    std::fs::write(
        temp_dir.path().join("tests").join("domain.rs"),
        format!("fn assertion_fixture(value: Option<u8>) -> u8 {{ value.{assertion_marker}) }}\n"),
    )?;

    let report = SourceQualityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(report.is_success(), "{}", report.to_cli_text());

    let unfinished_marker = marker_token("todo", '!');
    std::fs::write(
        temp_dir.path().join("tests").join("domain.rs"),
        format!("fn unfinished_fixture() {{ {unfinished_marker}(); }}\n"),
    )?;

    let report = SourceQualityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(!report.is_success());
    assert_eq!(report.finding_count, 1);
    assert_eq!(report.findings[0].category, "unfinished-implementation");

    std::fs::write(
        temp_dir.path().join("tests").join("domain.rs"),
        format!("fn assertion_fixture(value: Option<u8>) -> u8 {{ value.{assertion_marker}) }}\n"),
    )?;
    let oversized = (0..=MAX_RUST_SOURCE_LINES)
        .map(|index| format!("fn case_{index}() {{}}"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(temp_dir.path().join("tests").join("giant.rs"), oversized)?;

    let report = SourceQualityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(!report.is_success());
    assert_eq!(report.finding_count, 1);
    assert_eq!(report.findings[0].path, "tests/giant.rs");
    assert_eq!(report.findings[0].category, "oversized-rust-file");
    Ok(())
}
