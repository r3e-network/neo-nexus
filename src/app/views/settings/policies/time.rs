use super::super::super::super::{current_unix_time, text::format_optional_unix_age};

pub(super) fn time_fact(value: Option<u64>) -> String {
    current_unix_time().map_or_else(
        |_| value.map_or_else(|| "never".to_string(), |value| format!("unix {value}")),
        |now| format_optional_unix_age(value, now),
    )
}
