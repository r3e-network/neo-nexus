use std::path::Path;

#[test]
fn cli_help_source_uses_maintainable_structured_sections() -> anyhow::Result<()> {
    for relative in [
        "src/cli/actions/basics.rs",
        "src/cli/actions/basics/help.rs",
    ] {
        let help_source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let help_source = std::fs::read_to_string(help_source_path)?;
        let longest_line = help_source
            .lines()
            .map(str::len)
            .max()
            .ok_or_else(|| anyhow::anyhow!("help source should not be empty"))?;

        assert!(
            longest_line <= 240,
            "{relative} should use structured sections instead of very long lines"
        );
    }
    Ok(())
}
