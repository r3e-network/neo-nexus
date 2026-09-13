//! Headless API token management actions for CLI automation and CI/CD pipelines.

use std::str::FromStr;

use super::*;
use crate::core::security::TokenPermission;

/// `--list-api-tokens <db>`: print all API tokens in tabular format.
pub(in crate::cli::actions) fn list_api_tokens_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--list-api-tokens")?;
    let repository = Repository::open(PathBuf::from(&args[2]))
        .with_context(|| format!("failed to open workspace database {}", args[2]))?;

    let tokens = repository.list_api_tokens()?;
    if tokens.is_empty() {
        return Ok(CliAction::PrintWithExitCode {
            exit_code: 0,
            text: "No API tokens in workspace.".to_string(),
        });
    }

    let mut lines = Vec::with_capacity(tokens.len() + 1);
    lines.push(format!(
        "{:<36} {:<20} {:<24} {:<10}",
        "TOKEN ID", "NAME", "PERMISSIONS", "EXPIRED"
    ));
    for token in &tokens {
        let perms = token
            .permissions
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");
        lines.push(format!(
            "{:<36} {:<20} {:<24} {:<10}",
            token.id,
            token.name,
            perms,
            if token.is_expired() { "yes" } else { "no" }
        ));
    }

    Ok(CliAction::PrintWithExitCode {
        exit_code: 0,
        text: lines.join("\n"),
    })
}

/// `--list-api-tokens-json <db>`: print all API tokens as JSON.
pub(in crate::cli::actions) fn list_api_tokens_json_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 3, "--list-api-tokens-json")?;
    let repository = Repository::open(PathBuf::from(&args[2]))
        .with_context(|| format!("failed to open workspace database {}", args[2]))?;

    let tokens = repository.list_api_tokens()?;
    let json_tokens: Vec<serde_json::Value> = tokens
        .iter()
        .map(|token| {
            serde_json::json!({
                "id": token.id.to_string(),
                "name": token.name,
                "permissions": token.permissions.iter().map(|p| p.to_string()).collect::<Vec<_>>(),
                "created_at_unix": token.created_at_unix,
                "expires_at_unix": token.expires_at_unix,
                "is_expired": token.is_expired(),
            })
        })
        .collect();

    Ok(CliAction::PrintWithExitCode {
        exit_code: 0,
        text: serde_json::to_string_pretty(&json_tokens)?,
    })
}

/// `--create-api-token <db> <name> [permission]`: generate and store a new API token.
pub(in crate::cli::actions) fn create_api_token_action(args: &[String]) -> Result<CliAction> {
    if args.len() < 4 || args.len() > 5 {
        anyhow::bail!(
            "usage: neo-nexus --create-api-token <db> <name> [read_fleet|read_readiness|admin_all]"
        );
    }

    let repository = Repository::open(PathBuf::from(&args[2]))
        .with_context(|| format!("failed to open workspace database {}", args[2]))?;

    let name = args[3].trim();
    if name.is_empty() {
        anyhow::bail!("API token name cannot be empty");
    }

    let permissions = if let Some(raw_perms) = args.get(4) {
        let mut list = Vec::new();
        for item in raw_perms.split(',') {
            list.push(TokenPermission::from_str(item)?);
        }
        list
    } else {
        vec![TokenPermission::ReadFleet]
    };

    let (token, secret) = repository.create_api_token(name, permissions.clone(), None)?;

    let text = format!(
        "Created API token '{}' (ID: {})\nPermissions: {}\n\nBearer Token (save now - plaintext secret cannot be recovered):\n{}",
        token.name,
        token.id,
        permissions.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "),
        secret
    );

    Ok(CliAction::PrintWithExitCode { exit_code: 0, text })
}

/// `--revoke-api-token <db> <token-id>`: revoke an existing API token by UUID.
pub(in crate::cli::actions) fn revoke_api_token_action(args: &[String]) -> Result<CliAction> {
    require_arg_count(args, 4, "--revoke-api-token")?;
    let repository = Repository::open(PathBuf::from(&args[2]))
        .with_context(|| format!("failed to open workspace database {}", args[2]))?;

    let token_id = &args[3];
    let deleted = repository.delete_api_token(token_id)?;
    if deleted == 0 {
        anyhow::bail!("API token with ID '{token_id}' was not found in workspace");
    }

    Ok(CliAction::PrintWithExitCode {
        exit_code: 0,
        text: format!("Revoked API token '{token_id}'"),
    })
}
