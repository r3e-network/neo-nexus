use super::super::*;

pub(in crate::private_network) fn validate_signer_command(value: &str) -> Result<String> {
    let command = value.trim();
    if command.is_empty() {
        anyhow::bail!("signer sidecar command is empty");
    }
    if command.len() > 4096 {
        anyhow::bail!("signer sidecar command is too long");
    }
    if command
        .chars()
        .any(|character| character.is_control() && character != '\t')
    {
        anyhow::bail!("signer sidecar command contains control characters");
    }
    if command.contains('|') {
        anyhow::bail!("signer sidecar command must not contain pipe separators");
    }
    Ok(command.to_string())
}

pub(in crate::private_network) fn parse_signer_command_plan(
    command: &str,
) -> Result<SignerCommandPlan> {
    let command = validate_signer_command(command)?;
    let tokens = split_signer_command_tokens(&command)?;
    let Some((binary, arguments)) = tokens.split_first() else {
        anyhow::bail!("signer sidecar command plan is empty");
    };
    let plan = SignerCommandPlan {
        execution_policy: "argv-no-shell".to_string(),
        binary: binary.clone(),
        arguments: arguments.to_vec(),
    };
    validate_signer_command_plan(&plan)?;
    Ok(plan)
}

fn split_signer_command_tokens(command: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_token = false;
    let mut quote = None;

    for character in command.chars() {
        if let Some(quote_character) = quote {
            if character == quote_character {
                quote = None;
            } else {
                current.push(character);
            }
            continue;
        }

        match character {
            '\'' | '"' => {
                quote = Some(character);
                in_token = true;
            }
            character if character.is_whitespace() => {
                if in_token {
                    tokens.push(std::mem::take(&mut current));
                    in_token = false;
                }
            }
            _ => {
                current.push(character);
                in_token = true;
            }
        }
    }

    if let Some(quote_character) = quote {
        anyhow::bail!("signer sidecar command has an unclosed {quote_character} quote");
    }
    if in_token {
        tokens.push(current);
    }
    if tokens.is_empty() {
        anyhow::bail!("signer sidecar command plan is empty");
    }
    Ok(tokens)
}

pub(in crate::private_network) fn validate_signer_command_plan(
    plan: &SignerCommandPlan,
) -> Result<()> {
    if plan.execution_policy != "argv-no-shell" {
        anyhow::bail!("signer sidecar command plan must use argv-no-shell execution policy");
    }
    validate_signer_command_token(&plan.binary, "binary")?;
    for (index, argument) in plan.arguments.iter().enumerate() {
        validate_signer_command_token(argument, &format!("argument {}", index + 1))?;
    }
    Ok(())
}

fn validate_signer_command_token(value: &str, label: &str) -> Result<()> {
    if value.is_empty() {
        anyhow::bail!("signer sidecar command plan {label} is empty");
    }
    if value.len() > 4096 {
        anyhow::bail!("signer sidecar command plan {label} is too long");
    }
    if value.chars().any(|character| character.is_control()) {
        anyhow::bail!("signer sidecar command plan {label} contains control characters");
    }
    Ok(())
}

pub(in crate::private_network) fn signer_command_plan_matches_command(
    plan: &SignerCommandPlan,
    command: &str,
) -> bool {
    parse_signer_command_plan(command).is_ok_and(|parsed| parsed == *plan)
}
