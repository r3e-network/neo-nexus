use std::collections::BTreeSet;

use serde_yaml::{Mapping, Value};

pub(super) fn collect_matrix_os(yaml: &Value) -> Vec<String> {
    let mut found = BTreeSet::new();
    if let Some(jobs) = mapping_value(yaml, "jobs").and_then(Value::as_mapping) {
        for job in jobs.values() {
            if let Some(matrix) = mapping_value(job, "strategy")
                .and_then(|strategy| mapping_value(strategy, "matrix"))
            {
                if let Some(os) = mapping_value(matrix, "os") {
                    collect_os_values(os, &mut found);
                }
            }
        }
    }
    found.into_iter().collect()
}

fn collect_os_values(value: &Value, found: &mut BTreeSet<String>) {
    match value {
        Value::Sequence(items) => {
            for item in items {
                if let Some(os) = item.as_str() {
                    found.insert(os.to_string());
                }
            }
        }
        Value::String(os) => {
            found.insert(os.clone());
        }
        _ => {}
    }
}

fn mapping_value<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value
        .as_mapping()
        .and_then(|mapping| mapping_key_value(mapping, key))
}

fn mapping_key_value<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Value> {
    mapping
        .iter()
        .find_map(|(candidate, value)| (candidate.as_str() == Some(key)).then_some(value))
}
