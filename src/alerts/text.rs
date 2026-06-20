pub(super) fn truncate_for_message(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        value.to_string()
    } else {
        format!("{}...", value.chars().take(limit).collect::<String>())
    }
}
