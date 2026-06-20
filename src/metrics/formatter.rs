pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = UNITS[0];

    for next_unit in UNITS.iter().skip(1) {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = next_unit;
    }

    if unit == "B" {
        format!("{bytes} B")
    } else if value >= 10.0 {
        format!("{value:.0} {unit}")
    } else {
        format!("{value:.1} {unit}")
    }
}

pub(super) fn percent(part: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        clean_usage_percent(((part as f64 / total as f64) * 100.0) as f32)
    }
}

pub(super) fn clean_usage_percent(percent: f32) -> f32 {
    if !percent.is_finite() || percent <= 0.0 || percent.abs() < 0.05 {
        0.0
    } else {
        percent
    }
}
