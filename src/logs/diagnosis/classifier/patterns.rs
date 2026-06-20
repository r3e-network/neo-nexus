pub(super) fn is_port_binding_failure(value: &str) -> bool {
    contains_any(
        value,
        &[
            "address already in use",
            "addrinuse",
            "only one usage of each socket address",
            "port already in use",
        ],
    ) || (value.contains("bind") && value.contains("in use"))
}

pub(super) fn is_database_lock(value: &str) -> bool {
    (value.contains("rocksdb") || value.contains("leveldb") || value.contains("database"))
        && contains_any(value, &["lock", "locked", "already open", "resource busy"])
}

pub(super) fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}
