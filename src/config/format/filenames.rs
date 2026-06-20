use crate::types::{NodeConfig, NodeType};

use super::ConfigFormat;

pub(in crate::config) fn config_filename(node: &NodeConfig) -> String {
    format!(
        "{}-{}-config.{}",
        safe_filename_component(&node.name, &node.id),
        node.node_type,
        config_format(node.node_type).extension()
    )
}

pub(in crate::config) fn config_format(node_type: NodeType) -> ConfigFormat {
    match node_type {
        NodeType::NeoCli => ConfigFormat::Json,
        NodeType::NeoGo => ConfigFormat::Yaml,
        NodeType::NeoRs => ConfigFormat::Toml,
    }
}

fn safe_filename_component(value: &str, fallback: &str) -> String {
    let mut output = String::new();
    let mut last_was_dash = false;

    for character in value.trim().chars() {
        let next = if character.is_ascii_alphanumeric() || character == '_' {
            Some(character.to_ascii_lowercase())
        } else if character == '-' || character.is_whitespace() {
            Some('-')
        } else {
            None
        };

        if let Some(character) = next {
            if character == '-' {
                if !last_was_dash && !output.is_empty() {
                    output.push(character);
                    last_was_dash = true;
                }
            } else {
                output.push(character);
                last_was_dash = false;
            }
        }

        if output.len() >= 48 {
            break;
        }
    }

    while output.ends_with('-') {
        output.pop();
    }

    if output.is_empty() {
        safe_filename_component(fallback, "node")
    } else {
        output
    }
}
