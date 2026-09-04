//! Official Neo SignClient settings. Key selection and caller credentials belong
//! to the separately configured loopback bridge, never this node-side file.
use crate::{
    catalog::PluginId,
    types::{NodeConfig, NodeType},
};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::{
    net::IpAddr,
    path::{Path, PathBuf},
};

pub struct SignClientSettings {
    pub name: String,
    pub endpoint: String,
}

impl Default for SignClientSettings {
    fn default() -> Self {
        Self {
            name: "SignClient".into(),
            endpoint: "http://127.0.0.1:9991".into(),
        }
    }
}

impl SignClientSettings {
    pub fn read_for_node(work: &Path) -> Result<Self> {
        let path = config_path(work);
        if !path.is_file() {
            return Ok(Self::default());
        }
        let content: Value =
            serde_json::from_str(&without_comments(&std::fs::read_to_string(path)?))?;
        let section = &content["PluginConfiguration"];
        let settings = Self {
            name: section["Name"]
                .as_str()
                .context("missing SignClient name")?
                .into(),
            endpoint: section["Endpoint"]
                .as_str()
                .context("missing SignClient endpoint")?
                .into(),
        };
        settings.validate()?;
        Ok(settings)
    }

    pub fn configuration(&self) -> Value {
        json!({"PluginConfiguration": {"Name": self.name, "Endpoint": self.endpoint}})
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty()
            || self.name.len() > 64
            || !self
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        {
            anyhow::bail!("signer name must use 1-64 letters, digits, underscores or hyphens");
        }
        let endpoint = url::Url::parse(&self.endpoint).context("invalid SignClient endpoint")?;
        let loopback = endpoint.host_str().is_some_and(|host| {
            host == "localhost"
                || host
                    .trim_matches(['[', ']'])
                    .parse::<IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        });
        if endpoint.scheme() != "http"
            || !loopback
            || endpoint.port().is_none()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || endpoint.path() != "/"
        {
            anyhow::bail!("SignClient requires an HTTP loopback host with explicit bridge port, without credentials, path or query");
        }
        Ok(())
    }

    pub fn write_for_node(&self, work: &Path, node: &NodeConfig) -> Result<Option<PathBuf>> {
        if node.node_type != NodeType::NeoCli {
            anyhow::bail!("SignClient configuration applies to neo-cli only");
        }
        self.validate()?;
        let path = config_path(work);
        let generated = serde_json::to_string_pretty(&Self::default().configuration())? + "\n";
        // Preserve additional upstream/operator fields while changing only the
        // two documented settings. Invalid files are left untouched for review.
        let mut content = if path.is_file() {
            serde_json::from_str::<Value>(&without_comments(&std::fs::read_to_string(&path)?))
                .context(
                "existing SignClient config is not JSON; review it before changing the endpoint",
            )?
        } else {
            self.configuration()
        };
        let section = content
            .get_mut("PluginConfiguration")
            .and_then(Value::as_object_mut)
            .context("SignClient config requires a PluginConfiguration object")?;
        section.insert("Name".into(), self.name.clone().into());
        section.insert("Endpoint".into(), self.endpoint.clone().into());
        let custom = serde_json::to_string_pretty(&content)? + "\n";
        crate::config::write_config_override(
            &path,
            generated.as_bytes(),
            custom.as_bytes(),
            &node.runtime_version,
        )
    }
}

fn config_path(work: &Path) -> PathBuf {
    let disabled = super::activation::disabled_path(work, PluginId::SignClient);
    let directory = if disabled.is_dir() {
        disabled
    } else {
        work.join("Plugins/SignClient")
    };
    directory.join("SignClient.json")
}

// The upstream SignClient.json contains a // comment. Remove comments only
// outside quoted strings so URLs and operator values remain byte-for-byte intact.
fn without_comments(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    let mut quoted = false;
    let mut escaped = false;
    while let Some(character) = chars.next() {
        if quoted {
            result.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
        } else if character == '"' {
            quoted = true;
            result.push(character);
        } else if character == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for comment in chars.by_ref() {
                if comment == '\n' {
                    result.push('\n');
                    break;
                }
            }
        } else if character == '/' && chars.peek() == Some(&'*') {
            chars.next();
            while let Some(comment) = chars.next() {
                if comment == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
                if comment == '\n' {
                    result.push('\n');
                }
            }
            result.push(' ');
        } else {
            result.push(character);
        }
    }
    result
}

#[cfg(test)]
#[path = "../../tests/unit/plugins/signclient.rs"]
mod tests;
