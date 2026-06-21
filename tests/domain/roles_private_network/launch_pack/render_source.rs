const MAX_RENDER_SOURCE_LINE_LEN: usize = 240;

const PRIVATE_NETWORK_RENDER_SOURCES: &[(&str, &str)] = &[
    (
        "src/private_network/render/documents.rs",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/private_network/render/documents.rs"
        )),
    ),
    (
        "src/private_network/render/runbook_sections.rs",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/private_network/render/runbook_sections.rs"
        )),
    ),
    (
        "src/private_network/render/wallets.rs",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/private_network/render/wallets.rs"
        )),
    ),
];

#[test]
fn private_network_render_sources_keep_text_fragments_readable() {
    let offenders = PRIVATE_NETWORK_RENDER_SOURCES
        .iter()
        .flat_map(|(path, source)| {
            source
                .lines()
                .enumerate()
                .filter(|(_, line)| line.len() > MAX_RENDER_SOURCE_LINE_LEN)
                .map(move |(index, line)| format!("{path}:{} has {} chars", index + 1, line.len()))
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "private network render source lines are too long:\n{}",
        offenders.join("\n")
    );
}
