use serde_json::Value;

pub(in crate::config::validation::checks) fn json_path<'a>(
    value: &'a Value,
    path: &[&str],
) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

pub(in crate::config::validation::checks) fn yaml_path<'a>(
    value: &'a serde_yaml::Value,
    path: &[&str],
) -> Option<&'a serde_yaml::Value> {
    let mut current = value;
    for segment in path {
        let serde_yaml::Value::Mapping(mapping) = current else {
            return None;
        };
        current = mapping.get(serde_yaml::Value::String((*segment).to_string()))?;
    }
    Some(current)
}

pub(in crate::config::validation::checks) fn toml_path<'a>(
    value: &'a toml::Value,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

pub(in crate::config::validation::checks) fn dotted_path(path: &[&str]) -> String {
    path.join(".")
}
