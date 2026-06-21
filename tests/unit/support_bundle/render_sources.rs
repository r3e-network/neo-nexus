use std::path::Path;

#[test]
fn support_bundle_text_renderers_use_maintainable_structured_sections() -> anyhow::Result<()> {
    for relative in [
        "src/support_bundle/model/export.rs",
        "src/support_bundle/render/readme.rs",
        "src/support_bundle/render/privacy.rs",
    ] {
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let source = std::fs::read_to_string(source_path)?;
        let longest_line =
            source.lines().map(str::len).max().ok_or_else(|| {
                anyhow::anyhow!("support bundle renderer source should not be empty")
            })?;

        assert!(
            longest_line <= 240,
            "{relative} should use structured sections instead of very long lines"
        );
    }
    Ok(())
}
