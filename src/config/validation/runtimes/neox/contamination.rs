//! The guard against a Neo N3 setting reaching a Neo X config.
//!
//! The two families share a workspace, a database and most of this codebase,
//! and their settings look superficially alike — both have a "network", both
//! have "seed"/"boot" peers, both have validators. But an N3 network magic in
//! an EVM config is not a wrong number, it is a category error: a Neo X node
//! that took `860833102` as its chain id would sign transactions replayable
//! nowhere, and one that dialled `seed1.neo.org:10333` would be speaking the
//! wrong wire protocol at a node that cannot answer.
//!
//! Neither client would reject these keys the way neo-go rejects unknown YAML:
//! Reth's config is `#[serde(default)]` all the way down, so an extra table is
//! silently dropped. So this check exists because the runtime will not
//! complain.

use super::super::super::model::ConfigValidationReport;

/// Table and key names that only ever belong to a Neo N3 config.
const N3_ONLY_KEYS: [&str; 8] = [
    "ProtocolConfiguration",
    "ApplicationConfiguration",
    "StandbyCommittee",
    "SeedList",
    "network_magic",
    "seed_nodes",
    "ValidatorsCount",
    "MillisecondsPerBlock",
];

/// Substrings that give away an N3 seed host or an N3 magic number.
const N3_ONLY_VALUES: [&str; 4] = ["seed1.neo.org", "seed1t5.neo.org", "860833102", "894710606"];

pub(super) fn check(report: &mut ConfigValidationReport, value: &toml::Value) {
    let rendered = value.to_string();

    let leaked_key = N3_ONLY_KEYS
        .iter()
        .find(|key| contains_key(value, key))
        .copied();
    match leaked_key {
        Some(key) => report.critical(
            "Chain family",
            format!("Neo N3 key `{key}` reached a Neo X config; Neo X has no such setting."),
        ),
        None => report.pass("Chain family", "No Neo N3 settings are present."),
    }

    let leaked_value = N3_ONLY_VALUES
        .iter()
        .find(|needle| rendered.contains(**needle))
        .copied();
    match leaked_value {
        Some(needle) => report.critical(
            "Chain identity",
            format!("Neo N3 value `{needle}` reached a Neo X config."),
        ),
        None => report.pass("Chain identity", "No Neo N3 seeds or magics are present."),
    }
}

/// Whether a key appears anywhere in the document, at any depth.
fn contains_key(value: &toml::Value, key: &str) -> bool {
    match value {
        toml::Value::Table(table) => table
            .iter()
            .any(|(name, nested)| name == key || contains_key(nested, key)),
        toml::Value::Array(items) => items.iter().any(|item| contains_key(item, key)),
        _ => false,
    }
}
