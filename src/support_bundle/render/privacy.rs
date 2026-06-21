use crate::support_bundle::PRIVACY_POLICY;

pub(in crate::support_bundle) fn render_privacy_note() -> String {
    [
        format!("privacy-policy: {PRIVACY_POLICY}"),
        privacy_section("Included:", INCLUDED_LINES),
        privacy_section("Excluded:", EXCLUDED_LINES),
    ]
    .join("\n\n")
}

fn privacy_section(heading: &str, entries: &[&str]) -> String {
    let mut lines = vec![heading.to_string()];
    lines.extend(entries.iter().map(|entry| format!("- {entry}")));
    lines.push(String::new());
    lines.join("\n")
}

const INCLUDED_LINES: &[&str] = &[
    "readiness report",
    "read-only SQLite integrity report",
    "system and managed node metrics snapshot",
    "bounded event journal report",
    "redacted node inventory",
    "redacted runtime log diagnosis summaries",
];

const EXCLUDED_LINES: &[&str] = &[
    "raw workspace database",
    "raw runtime logs",
    "private keys",
    "wallet passwords",
    "passphrases",
    "mnemonics and seeds",
    "authorization bearer values",
    "API keys and tokens",
    "webhook secrets",
    "cached runtime packages and snapshots",
];
