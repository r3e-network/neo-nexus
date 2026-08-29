//! Human-readable time for the workbench.
//!
//! Every timestamp the workspace stores is a Unix second, and rendering those
//! raw makes an operator do long division to find out when a node died. These
//! helpers print an absolute UTC time — the primary reading, because logs,
//! events and probes are all UTC — with the elapsed time beside it, which is
//! what you actually scan for when deciding whether something needs attention.
//!
//! The civil-date conversion is days-since-epoch arithmetic rather than a
//! dependency: the workbench keeps its binary self-contained.

use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the epoch, as the clock reads now.
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

/// `2026-08-29 10:09Z`. Deliberately fixed-width so a column of them aligns.
pub fn utc_label(unix: u64) -> String {
    let seconds = unix as i64;
    let days = seconds.div_euclid(86_400);
    let time_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}Z",
        time_of_day / 3_600,
        (time_of_day % 3_600) / 60,
    )
}

/// How long ago, rounded to a unit a person would say out loud.
pub fn relative(unix: u64, now: u64) -> String {
    let elapsed = now.saturating_sub(unix);
    // A timestamp a little in the future means clock skew, not the future.
    if unix > now {
        return "just now".to_string();
    }
    match elapsed {
        0..=4 => "just now".to_string(),
        5..=59 => format!("{elapsed}s ago"),
        60..=3_599 => format!("{}m ago", elapsed / 60),
        3_600..=86_399 => format!("{}h ago", elapsed / 3_600),
        _ => format!("{}d ago", elapsed / 86_400),
    }
}

/// A table cell holding an absolute time with its elapsed reading beside it.
/// Renders as plain text when the value is absent, so a column never shows a
/// bare zero for "never happened".
pub fn time_cell(unix: Option<u64>) -> String {
    let now = now_unix();
    match unix {
        Some(stamp) => format!(
            r#"<time datetime="{iso}">{label}<span class="elapsed">{ago}</span></time>"#,
            iso = iso8601(stamp),
            label = utc_label(stamp),
            ago = relative(stamp, now),
        ),
        None => "—".to_string(),
    }
}

/// The same value as a machine-readable attribute for browsers and tools.
fn iso8601(unix: u64) -> String {
    let seconds = unix as i64;
    let days = seconds.div_euclid(86_400);
    let time_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time_of_day / 3_600,
        (time_of_day % 3_600) / 60,
        time_of_day % 60,
    )
}

/// Days since 1970-01-01 to a proleptic Gregorian date.
///
/// The shift to a March-based year is what removes the usual February
/// special case: the extra day lands at the end of the sequence instead of the
/// middle.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_position = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_position + 2) / 5 + 1) as u32;
    let month = if month_position < 10 {
        (month_position + 3) as u32
    } else {
        (month_position - 9) as u32
    };
    if month <= 2 {
        year += 1;
    }
    (year, month, day)
}

#[cfg(test)]
#[path = "../../tests/unit/web/time/tests.rs"]
mod tests;
