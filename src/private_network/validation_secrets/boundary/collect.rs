use super::{
    keys::wallet_provisioning_sensitive_key, strings::wallet_provisioning_sensitive_string,
};

pub(super) fn collect_wallet_provisioning_secret_findings(
    value: &serde_json::Value,
    path: &str,
    findings: &mut Vec<String>,
) {
    match value {
        serde_json::Value::Object(map) => collect_object_findings(map, path, findings),
        serde_json::Value::Array(values) => collect_array_findings(values, path, findings),
        serde_json::Value::String(text) => collect_string_finding(text, path, findings),
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}

fn collect_object_findings(
    map: &serde_json::Map<String, serde_json::Value>,
    path: &str,
    findings: &mut Vec<String>,
) {
    for (key, child) in map {
        let child_path = child_object_path(path, key);
        if wallet_provisioning_sensitive_key(key) {
            push_secret_finding(findings, child_path.clone());
        }
        collect_wallet_provisioning_secret_findings(child, &child_path, findings);
    }
}

fn collect_array_findings(values: &[serde_json::Value], path: &str, findings: &mut Vec<String>) {
    for (index, child) in values.iter().enumerate() {
        collect_wallet_provisioning_secret_findings(child, &format!("{path}[{index}]"), findings);
    }
}

fn collect_string_finding(text: &str, path: &str, findings: &mut Vec<String>) {
    if wallet_provisioning_sensitive_string(text) {
        push_secret_finding(findings, path.to_string());
    }
}

fn child_object_path(path: &str, key: &str) -> String {
    if path == "$" {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

fn push_secret_finding(findings: &mut Vec<String>, path: String) {
    if !findings.iter().any(|finding| finding == &path) {
        findings.push(path);
    }
}
