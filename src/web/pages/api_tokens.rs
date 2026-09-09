//! API Token Management Page
//!
//! Allows operators to create, list, and delete API authentication tokens for
//! programmatic access to protected endpoints via Bearer token headers.

use axum::{
    body::Body,
    extract::{Form, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use crate::wallet::TokenPermission;
use crate::web::WebState;

pub async fn api_tokens_page(State(state): State<WebState>) -> Response {
    let Some(body) = render_body(&state.repository).ok() else {
        return Response::builder()
            .status(500)
            .body(Body::from("Internal Server Error"))
            .unwrap();
    };

    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>API Tokens - Settings | NeoNexus</title>
    <link rel="stylesheet" href="/styles/main.css">
</head>
<body class="settings-page">
    <nav class="breadcrumbs">
        <a href="/settings">Settings</a> / API Tokens
    </nav>
    <main>
{body}
    </main>
</body>
</html>"#,
        body = body
    ))
    .into_response()
}

/// Form POST handler for creating new API tokens
pub async fn create_token(
    State(state): State<WebState>,
    Form(form_data): Form<TokenCreateForm>,
) -> Result<Response, (StatusCode, String)> {
    let name = form_data.name.trim();

    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Token name is required".to_string(),
        ));
    }

    // Build permissions from form data
    let mut permissions = Vec::new();

    if form_data.read_fleet {
        permissions.push(TokenPermission::ReadFleet);
    }
    if form_data.read_readiness {
        permissions.push(TokenPermission::ReadReadiness);
    }
    if form_data.admin_all {
        permissions.push(TokenPermission::AdminAll);
    }

    // Require at least one permission
    if permissions.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Select at least one permission".to_string(),
        ));
    }

    // Create the token (no expiration for now)
    match state.repository.create_api_token(name, permissions, None) {
        Ok((token, secret)) => {
            let body = render_with_created_token(&state.repository, &token, &secret);
            Ok(Html(body).into_response())
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error creating token: {}", e),
        )),
    }
}

/// POST handler for deleting a token
pub async fn delete_token(
    state: State<WebState>,
    path: Path<String>,
) -> Result<Response, (StatusCode, String)> {
    match state.repository.delete_api_token(&path.0) {
        Ok(_) => {
            let body = "Token deleted. <a href='/settings/api-tokens'>Back to tokens</a>";
            Ok(Html(body.to_string()).into_response())
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error deleting token: {}", e),
        )),
    }
}

#[derive(serde::Deserialize)]
pub struct TokenCreateForm {
    name: String,
    read_fleet: bool,
    read_readiness: bool,
    admin_all: bool,
}

fn render_body(repository: &crate::repository::Repository) -> anyhow::Result<String> {
    let tokens = repository.list_api_tokens()?;
    let token_table = render_token_table(&tokens);

    Ok(format!(
        r#"<h1>API Tokens</h1>
<p class="muted">Create and manage API authentication tokens for programmatic access to NeoNexus endpoints.</p>
<div class="grid">
    <section class="card">
        <h2>Create New Token</h2>
        {creation_form}
    </section>
    <section class="card">
        <h2>Existing Tokens</h2>
        {token_table}
    </section>
</div>"#,
        creation_form = create_token_form(),
        token_table = token_table
    ))
}

fn render_with_created_token(
    repository: &crate::repository::Repository,
    token: &crate::wallet::ApiToken,
    secret: &str,
) -> String {
    let tokens = repository.list_api_tokens().unwrap_or_default();
    let token_table = render_token_table(&tokens);

    format!(
        "<h1>API Tokens</h1>\n<div class=\"success-box\">\n    <h2>Token Created Successfully!</h2>\n    <p><strong>Your generated token (copy it NOW - cannot retrieve later):</strong></p>\n    <div class=\"token-display\">\n        <code>{}</code>\n    </div>\n    <p class=\"warn\">This secret will NEVER be shown again. Save it securely!</p>\n    <button onclick=\"navigator.clipboard.writeText('{}'); alert('Copied!')\">Copy to Clipboard</button>\n</div>\n<div class=\"grid\">\n    <section class=\"card\">\n        <h2>Token Details</h2>\n{}\n    </section>\n    <section class=\"card\">\n        <h2>Existing Tokens</h2>\n{}\n    </section>\n</div>",
        escape_html(secret),
        escape_html(secret),
        render_token_details(token),
        token_table
    )
}

fn render_token_details(token: &crate::wallet::ApiToken) -> String {
    let id_display = token.display_id();
    let created_at = pretty_timestamp(token.created_at_unix);
    let expires = token
        .expires_at_unix
        .map_or_else(|| "Never".to_string(), pretty_timestamp);

    let permissions_str = token
        .permissions
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let status_class = if token.is_expired() {
        "expired"
    } else {
        "active"
    };

    format!(
        r#"<dl class="token-meta">
    <dt>ID</dt>
    <dd><code>{id_display}</code></dd>
    
    <dt>Name</dt>
    <dd>{name}</dd>
    
    <dt>Created</dt>
    <dd>{created_at}</dd>
    
    <dt>Expires</dt>
    <dd>{expires}</dd>
    
    <dt>Permissions</dt>
    <dd>{permissions_str}</dd>
    
    <dt>Status</dt>
    <dd class="{status_class}">{status}</dd>
</dl>"#,
        id_display = escape_html(&id_display),
        name = escape_html(&token.name),
        created_at = escape_html(&created_at),
        expires = escape_html(&expires),
        permissions_str = escape_html(&permissions_str),
        status_class = status_class,
        status = if token.is_expired() {
            "Expired"
        } else {
            "Active"
        }
    )
}

fn create_token_form() -> String {
    r#"<form method="post" action="/settings/api-tokens/create" class="filters">
    <div class="form-group">
        <label for="token-name">Token Name</label>
        <input type="text" id="token-name" name="name" placeholder="e.g., CI/CD Pipeline" required>
        <small>Give this token a memorable name for identification</small>
    </div>
    
    <div class="form-group">
        <label>Permissions</label>
        <label class="checkbox-label">
            <input type="checkbox" name="read_fleet"> Read Fleet Access
        </label>
        <label class="checkbox-label">
            <input type="checkbox" name="read_readiness"> Read Readiness Access
        </label>
        <label class="checkbox-label">
            <input type="checkbox" name="admin_all"> Full Admin Access
        </label>
        <small>Select at least one permission</small>
    </div>
    
    <button type="submit" class="primary-btn">Generate Token</button>
</form>"#
        .to_string()
}

fn render_token_table(tokens: &[crate::wallet::ApiToken]) -> String {
    if tokens.is_empty() {
        return "<p class='muted'>No API tokens have been created yet.</p>".to_string();
    }

    let rows: Vec<String> = tokens
        .iter()
        .map(|token| {
            let id_display = token.display_id();
            let name = escape_html(&token.name);
            let created_at = pretty_timestamp(token.created_at_unix);
            let expires = token.expires_at_unix.map_or_else(
                || "Never".to_string(),
                pretty_timestamp
            );
            let permissions_str = token.permissions.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ");

            let status_class = if token.is_expired() {
                "badge expired"
            } else {
                "badge active"
            };
            let status_text = if token.is_expired() { "Expired" } else { "Active" };

            format!(
                r#"<tr>
    <td><strong>{name}</strong></td>
    <td><code>{id_display}</code></td>
    <td>{created_at}</td>
    <td>{expires}</td>
    <td><span class="badges">{perms}</span></td>
    <td><span class="{status_class}">{status}</span></td>
    <td>
        <form method="POST" action="/settings/api-tokens/{token_id}/delete" class="inline-form">
            <button type="submit" class="danger-btn" onclick="return confirm('Delete token {name}?')">Delete</button>
        </form>
    </td>
</tr>"#,
                name = name,
                id_display = id_display,
                created_at = created_at,
                expires = expires,
                perms = permissions_str,
                status_class = status_class,
                status = status_text,
                token_id = token.id
            )
        })
        .collect();

    let rows_html = rows.join("\n");

    format!(
        r#"<div class="table-container">
<table class="data-table">
    <thead>
        <tr>
            <th>Name</th>
            <th>ID</th>
            <th>Created</th>
            <th>Expires</th>
            <th>Permissions</th>
            <th>Status</th>
            <th>Actions</th>
        </tr>
    </thead>
    <tbody>
        {rows}
    </tbody>
</table>
</div>"#,
        rows = rows_html
    )
}

fn pretty_timestamp(unix_ts: i64) -> String {
    // Simple formatting - could use chrono for better date handling
    unix_ts.to_string()
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
