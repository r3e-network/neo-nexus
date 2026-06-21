use super::*;

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
fn source_quality_rejects_oversized_maintenance_files() -> anyhow::Result<()> {
    assert_eq!(MAX_MAINTENANCE_FILE_LINES, 1000);
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join("docs"))?;
    std::fs::create_dir_all(temp_dir.path().join(".github").join("workflows"))?;
    std::fs::write(
        temp_dir.path().join("README.md"),
        oversized_lines("runbook line", MAX_MAINTENANCE_FILE_LINES),
    )?;
    std::fs::write(
        temp_dir.path().join("docs").join("catalog.json"),
        (0..=MAX_MAINTENANCE_FILE_LINES)
            .map(|index| format!("{{\"line\": {index}}}"))
            .collect::<Vec<_>>()
            .join("\n"),
    )?;
    std::fs::write(
        temp_dir
            .path()
            .join(".github")
            .join("workflows")
            .join("ci.yml"),
        oversized_lines("# workflow line", MAX_MAINTENANCE_FILE_LINES),
    )?;
    let report = SourceQualityChecker::check_at(temp_dir.path(), 1_800_000_000)?;
    assert!(!report.is_success());
    assert_eq!(report.finding_count, 3);
    assert!(report.findings.iter().any(|finding| {
        finding.path == "README.md" && finding.category == "oversized-maintenance-file"
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.path == "docs/catalog.json" && finding.category == "oversized-maintenance-file"
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.path == ".github/workflows/ci.yml"
            && finding.category == "oversized-maintenance-file"
    }));
    Ok(())
}

#[test]
fn source_quality_rejects_oversized_named_maintenance_files() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::write(
        temp_dir.path().join("Makefile"),
        oversized_lines("verify:", MAX_MAINTENANCE_FILE_LINES),
    )?;
    std::fs::write(
        temp_dir.path().join("LICENSE"),
        oversized_lines("license line", MAX_MAINTENANCE_FILE_LINES),
    )?;
    let report = SourceQualityChecker::check_at(temp_dir.path(), 1_800_000_000)?;
    assert!(!report.is_success());
    assert_eq!(report.finding_count, 2);
    assert!(report.findings.iter().any(|finding| {
        finding.path == "Makefile" && finding.category == "oversized-maintenance-file"
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.path == "LICENSE" && finding.category == "oversized-maintenance-file"
    }));
    Ok(())
}

#[test]
fn source_quality_rejects_maintenance_files_case_insensitively() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::create_dir_all(temp_dir.path().join(".github").join("workflows"))?;
    std::fs::write(
        temp_dir.path().join("README.MD"),
        oversized_lines("readme line", MAX_MAINTENANCE_FILE_LINES),
    )?;
    std::fs::write(
        temp_dir
            .path()
            .join(".github")
            .join("workflows")
            .join("CI.YAML"),
        oversized_lines("workflow line", MAX_MAINTENANCE_FILE_LINES),
    )?;
    std::fs::write(
        temp_dir.path().join("makefile"),
        oversized_lines("verify:", MAX_MAINTENANCE_FILE_LINES),
    )?;
    std::fs::write(
        temp_dir.path().join("notice"),
        oversized_lines("notice line", MAX_MAINTENANCE_FILE_LINES),
    )?;
    let report = SourceQualityChecker::check_at(temp_dir.path(), 1_800_000_000)?;
    assert!(!report.is_success());
    assert_eq!(report.finding_count, 4);
    for path in [
        ".github/workflows/CI.YAML",
        "README.MD",
        "makefile",
        "notice",
    ] {
        assert!(report.findings.iter().any(|finding| {
            finding.path == path && finding.category == "oversized-maintenance-file"
        }));
    }
    Ok(())
}

fn oversized_lines(prefix: &str, limit: usize) -> String {
    (0..=limit)
        .map(|index| format!("{prefix} {index}"))
        .collect::<Vec<_>>()
        .join("\n")
}
