use crate::support_bundle::PRIVACY_POLICY;

pub(in crate::support_bundle) fn render_privacy_note() -> String {
    format!(
        "privacy-policy: {PRIVACY_POLICY}\n\nIncluded:\n- readiness report\n- read-only SQLite integrity report\n- system and managed node metrics snapshot\n- bounded event journal report\n- redacted node inventory\n- redacted runtime log diagnosis summaries\n\nExcluded:\n- raw workspace database\n- raw runtime logs\n- private keys\n- wallet passwords\n- passphrases\n- mnemonics and seeds\n- authorization bearer values\n- API keys and tokens\n- webhook secrets\n- cached runtime packages and snapshots\n"
    )
}
