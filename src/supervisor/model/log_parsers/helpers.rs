use super::{LogEntry, SyncProgress};

pub(super) fn contains_any_case(haystack: &str, needles: &[&str]) -> bool {
    let lower = haystack.to_lowercase();
    needles.iter().any(|needle| lower.contains(needle))
}

pub(super) fn extract_bracket_number(line: &str, keyword: &str) -> Option<u64> {
    let lower = line.to_lowercase();
    if let Some(idx) = lower.find(keyword) {
        let from_idx = idx + keyword.len();
        let rest = line[from_idx..].trim();
        let number_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        number_str.parse::<u64>().ok()
    } else {
        None
    }
}

pub(super) fn ts_parse_timestamp(ts_str: &str) -> Option<u64> {
    // YYYY/MM/DD HH:MM:SS.mmm format
    let cleaned = ts_str.replace('/', "-").replace(' ', "T");
    if let Some(idx) = cleaned.find('+') {
        let ts_part = &cleaned[..idx];
        if ts_part.len() >= 19 && ts_part.contains('T') {
            return unix_epoch_approximation(ts_part);
        }
    } else if cleaned.len() >= 19 {
        return unix_epoch_approximation(&cleaned[..19.min(cleaned.len())]);
    }

    ts_str
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<u64>()
        .ok()
        .filter(|&seconds| seconds > 1_000_000_000)
}

pub(super) fn unix_epoch_approximation(iso_like: &str) -> Option<u64> {
    iso_like
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(13)
        .collect::<String>()
        .parse::<u64>()
        .ok()
}

pub(super) fn extract_neogo_height(line: &str) -> Option<u64> {
    if let Some(start) = line.find("blockHeight=") {
        let rest = &line[start + 12..];
        let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        return num.parse().ok();
    }
    if let Some(start) = line.find("Block #") {
        let rest = &line[start + 7..];
        let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        return num.parse().ok();
    }
    None
}

pub(super) fn extract_geth_block_height(line: &str) -> Option<SyncProgress> {
    if line.contains("Chain imported") || line.contains("importing blocks") {
        let current = extract_bracket_number(line, "block=")?;
        let target =
            extract_bracket_number(line, "of").or_else(|| extract_bracket_number(line, "total"))?;
        measured_sync_progress(current, target, extract_peer_count(line))
    } else {
        None
    }
}

pub(super) fn extract_peer_count(line: &str) -> u32 {
    if let Some(start) = line.find("peers=") {
        let rest = &line[start + 6..];
        rest.chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u32>()
            .unwrap_or(0)
    } else {
        0
    }
}

pub(super) fn measured_sync_progress(
    current: u64,
    target: u64,
    peers: u32,
) -> Option<SyncProgress> {
    if target == 0 || current > target {
        return None;
    }
    Some(SyncProgress {
        current_height: current,
        target_height: target,
        sync_percentage: (current as f64 / target as f64 * 100.0) as f32,
        peers_connected: peers,
    })
}

pub(super) fn parse_reth_json_entry(entry: &serde_json::Value) -> Option<LogEntry> {
    let ts = entry.get("time").and_then(|t| t.as_i64()).unwrap_or(0) as u64;
    let level = entry
        .get("target")
        .and_then(|l| l.as_str())
        .unwrap_or("reth")
        .split(':')
        .next()
        .unwrap_or("info")
        .to_uppercase();
    let msg = entry.get("message").and_then(|m| m.as_str()).unwrap_or("");

    Some(LogEntry {
        timestamp: ts.max(1),
        level,
        message: msg.to_string(),
        source: None,
        metadata: entry.clone(),
    })
}

pub(super) fn find_log_level(line: &str) -> String {
    let level_char = line
        .chars()
        .find(|c| matches!(c, 'I' | 'D' | 'W' | 'E' | 'F' | 'i' | 'd' | 'w' | 'e' | 'f'));

    let level_upper = match level_char {
        Some('I' | 'i') => "INFO",
        Some('D' | 'd') => "DEBUG",
        Some('W' | 'w') => "WARN",
        Some('E' | 'e') => "ERROR",
        Some('F' | 'f') => "FATAL",
        _ => "INFO",
    };

    level_upper.to_string()
}

pub(super) fn extract_iso_timestamp(line: &str) -> Option<u64> {
    if let Some(idx) = line.find('T') {
        let iso_part = &line[idx..];
        if let Some(end) = iso_part
            .find('+')
            .or_else(|| iso_part.find('-').filter(|&e| e > 10))
        {
            let ts_str = &iso_part[..end];
            if ts_str.len() >= 19 {
                return unix_epoch_approximation(ts_str);
            }
        }
    }
    None
}

pub(super) fn extract_kv_value<'a>(haystack: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("{}=", key);
    if let Some(start) = haystack.find(&needle) {
        let val_start = start + key.len() + 1;
        let rest = &haystack[val_start..];
        let end = rest.find(' ').unwrap_or(rest.len());
        Some(&rest[..end])
    } else {
        None
    }
}

pub(super) fn extract_reth_height(line: &str) -> Option<u64> {
    if let Some(start) = line.find('#') {
        let rest = &line[start + 1..];
        rest.chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u64>()
            .ok()
    } else {
        None
    }
}
