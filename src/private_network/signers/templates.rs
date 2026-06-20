use super::{super::*, commands::validate_signer_command};

pub(in crate::private_network) fn validate_signer_command_template(value: &str) -> Result<String> {
    let template = validate_signer_command(value)?;
    validate_signer_template_syntax(&template)?;
    Ok(template)
}

fn validate_signer_template_syntax(template: &str) -> Result<()> {
    let mut chars = template.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '{' => {
                if chars.next_if_eq(&'{').is_some() {
                    continue;
                }
                let mut field = String::new();
                loop {
                    let Some(next) = chars.next() else {
                        anyhow::bail!(
                            "signer sidecar command template has an unclosed placeholder"
                        );
                    };
                    if next == '}' {
                        break;
                    }
                    field.push(next);
                }
                validate_signer_template_field(&field)?;
            }
            '}' => {
                if chars.next_if_eq(&'}').is_none() {
                    anyhow::bail!("signer sidecar command template has an unopened placeholder");
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_signer_template_field(field: &str) -> Result<()> {
    match field {
        "wallet" | "endpoint" | "label" | "public_key" => Ok(()),
        "" => anyhow::bail!("signer sidecar command template has an empty placeholder"),
        other => anyhow::bail!("unsupported signer sidecar command placeholder: {other}"),
    }
}

pub(in crate::private_network) fn expand_signer_command_template(
    template: &str,
    signer: &CommitteeSigner,
) -> Result<String> {
    let template = validate_signer_command_template(template)?;
    let mut expanded = String::new();
    let mut chars = template.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '{' => {
                if chars.next_if_eq(&'{').is_some() {
                    expanded.push('{');
                    continue;
                }
                let mut field = String::new();
                loop {
                    let Some(next) = chars.next() else {
                        anyhow::bail!(
                            "signer sidecar command template has an unclosed placeholder"
                        );
                    };
                    if next == '}' {
                        break;
                    }
                    field.push(next);
                }
                expanded.push_str(&signer_template_value(&field, signer)?);
            }
            '}' => {
                if chars.next_if_eq(&'}').is_some() {
                    expanded.push('}');
                } else {
                    anyhow::bail!("signer sidecar command template has an unopened placeholder");
                }
            }
            _ => expanded.push(character),
        }
    }
    validate_signer_command(&expanded)
}

fn signer_template_value(field: &str, signer: &CommitteeSigner) -> Result<String> {
    match field {
        "label" => Ok(signer.label.clone()),
        "public_key" => Ok(signer.public_key.clone()),
        "wallet" => signer
            .wallet_path
            .as_ref()
            .map(|path| path.display().to_string())
            .context("signer sidecar command template uses {wallet} without a wallet path"),
        "endpoint" => signer
            .signer_endpoint
            .clone()
            .context("signer sidecar command template uses {endpoint} without a signer endpoint"),
        _ => {
            validate_signer_template_field(field)?;
            unreachable!("validated signer template field was not handled")
        }
    }
}
