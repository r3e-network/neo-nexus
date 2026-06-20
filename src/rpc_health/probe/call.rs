use anyhow::{Context, Result};
use serde_json::{json, Value};

use super::summary::compact_json;

pub(super) fn call_method(
    agent: &ureq::Agent,
    endpoint: &str,
    method: &'static str,
) -> Result<Value> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": "neonexus-health",
        "method": method,
        "params": []
    });
    let response = agent
        .post(endpoint)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .with_context(|| format!("failed to call {method} at {endpoint}"))?;
    let text = response
        .into_string()
        .with_context(|| format!("failed to read {method} response"))?;
    let json: Value =
        serde_json::from_str(&text).with_context(|| format!("{method} returned invalid JSON"))?;
    if let Some(error) = json.get("error") {
        anyhow::bail!("{method} returned error: {}", compact_json(error));
    }
    json.get("result")
        .cloned()
        .with_context(|| format!("{method} response is missing result"))
}
