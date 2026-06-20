use std::path::Path;

pub(super) fn short_path(path: &Path, max_chars: usize) -> String {
    truncate_middle(&path.display().to_string(), max_chars)
}

pub(super) fn non_empty(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

pub(super) fn truncate_middle(value: &str, max_chars: usize) -> String {
    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value.to_string();
    }

    let keep = max_chars.saturating_sub(3);
    let head = keep / 2;
    let tail = keep.saturating_sub(head);
    let prefix: String = value.chars().take(head).collect();
    let suffix: String = value
        .chars()
        .rev()
        .take(tail)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{prefix}...{suffix}")
}

pub(super) fn truncate_end(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated: String = value.chars().take(max_chars.saturating_sub(3)).collect();
    truncated.push_str("...");
    truncated
}

pub(super) fn format_optional_unix_age(value: Option<u64>, now: u64) -> String {
    value.map_or_else(|| "never".to_string(), |value| format_unix_age(value, now))
}

pub(super) fn format_unix_age(value: u64, now: u64) -> String {
    if value <= now {
        let age = now - value;
        if age < 60 {
            "just now".to_string()
        } else {
            format!("{} ago", compact_duration(age))
        }
    } else {
        format!("in {}", compact_duration(value - now))
    }
}

fn compact_duration(seconds: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;

    if seconds < HOUR {
        format!("{} min", seconds / MINUTE)
    } else if seconds < DAY {
        format!("{} hr", seconds / HOUR)
    } else {
        format!("{} day", seconds / DAY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_middle_preserves_both_ends() {
        assert_eq!(
            truncate_middle("/very/long/path/to/file.db", 14),
            "/very...ile.db"
        );
    }

    #[test]
    fn truncate_end_preserves_prefix() {
        assert_eq!(truncate_end("abcdef", 4), "a...");
    }

    #[test]
    fn non_empty_replaces_blank_values() {
        assert_eq!(non_empty("   ", "-"), "-");
        assert_eq!(non_empty("value", "-"), "value");
    }

    #[test]
    fn unix_age_labels_are_human_readable() {
        assert_eq!(format_optional_unix_age(None, 10_000), "never");
        assert_eq!(format_unix_age(10_000, 10_000), "just now");
        assert_eq!(format_unix_age(9_940, 10_000), "1 min ago");
        assert_eq!(format_unix_age(2_800, 10_000), "2 hr ago");
        assert_eq!(format_unix_age(13_600, 10_000), "in 1 hr");
    }
}
