pub(super) fn summarize_secret_findings(findings: &[String]) -> String {
    let mut shown = findings
        .iter()
        .take(8)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    if findings.len() > 8 {
        shown.push_str(&format!(", +{} more", findings.len() - 8));
    }
    shown
}
