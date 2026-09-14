//! API Token Management Page
//!
//! Allows operators to create, list, and delete API authentication tokens for
//! programmatic access to protected endpoints via Bearer token headers.

use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::wallet::TokenPermission;
use crate::web::{html, WebState};

pub async fn api_tokens_page(
    State(state): State<WebState>,
    Query(params): Query<std::collections::BTreeMap<String, String>>,
) -> Response {
    let body = match render_body(&state.workspace) {
        Ok(b) => b,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    };
    let flash = params.get("flash").map(String::as_str).unwrap_or("");
    Html(html::layout("API Tokens", "settings", flash, &body)).into_response()
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

    if is_checked(form_data.read_fleet.as_deref()) {
        permissions.push(TokenPermission::ReadFleet);
    }
    if is_checked(form_data.read_readiness.as_deref()) {
        permissions.push(TokenPermission::ReadReadiness);
    }
    if is_checked(form_data.admin_all.as_deref()) {
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
    match state.commands.create_api_token(name, permissions, None) {
        Ok((token, secret)) => {
            let _ = state.commands.record_event(crate::events::NewRuntimeEvent {
                node_id: None,
                node_name: None,
                kind: crate::events::EventKind::ApiTokenCreated,
                severity: crate::events::EventSeverity::Info,
                message: format!("created API token '{}' ({})", token.name, token.id),
            });
            let body = render_with_created_token(&state.workspace, &token, &secret);
            Ok(Html(body).into_response())
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error creating token: {}", e),
        )),
    }
}

/// POST handler for deleting a token
pub async fn delete_token(State(state): State<WebState>, Path(token_id): Path<String>) -> Response {
    let outcome = (|| -> anyhow::Result<()> {
        state.commands.delete_api_token(&token_id)?;
        let _ = state.commands.record_event(crate::events::NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: crate::events::EventKind::ApiTokenDeleted,
            severity: crate::events::EventSeverity::Info,
            message: format!("deleted API token '{token_id}'"),
        });
        Ok(())
    })();

    let flash = match outcome {
        Ok(()) => format!("API token {token_id} deleted"),
        Err(e) => format!("failed to delete token: {e}"),
    };

    Redirect::to(&format!(
        "/settings/api-tokens?flash={}",
        html::urlencoding_lite(&flash)
    ))
    .into_response()
}

fn is_checked(val: Option<&str>) -> bool {
    match val {
        Some(s) => {
            let s = s.trim().to_ascii_lowercase();
            s == "on" || s == "true" || s == "1" || s == "yes"
        }
        None => false,
    }
}

#[derive(serde::Deserialize)]
pub struct TokenCreateForm {
    #[serde(default)]
    name: String,
    #[serde(default)]
    read_fleet: Option<String>,
    #[serde(default)]
    read_readiness: Option<String>,
    #[serde(default)]
    admin_all: Option<String>,
}

fn render_body(
    workspace: &crate::core::workspace_queries::WorkspaceQueries,
) -> anyhow::Result<String> {
    let tokens = workspace.list_api_tokens()?;
    let token_table = render_token_table(&tokens);
    let breadcrumb = html::breadcrumb(&[("NeoNexus", "/"), ("API tokens", "")]);
    let head = html::page_head(
        "IAM Security Credentials & Access Keys",
        "Manage programmatic access keys, secret tokens, and least-privilege IAM permission boundaries.",
        r#"<a class="btn" href="/signer">KMS Custody</a>"#,
    );

    Ok(format!(
        r#"{breadcrumb}
{head}
<div class="cards" style="margin-bottom: 20px;">
    <div class="card"><div class="stat-label">Active Access Keys</div><div class="stat-value">{}</div><div class="stat-detail">Programmatic service credentials</div></div>
    <div class="card"><div class="stat-label">Permission Enforcement</div><div class="stat-value" style="color: var(--jade);">Strict RBAC</div><div class="stat-detail">Signed Bearer token verification</div></div>
    <div class="card"><div class="stat-label">Cryptographic Storage</div><div class="stat-value" style="color: var(--cyan);">Argon2id</div><div class="stat-detail">Zero raw secrets in persistent store</div></div>
</div>
<div class="grid" style="grid-template-columns: minmax(300px, 1fr) minmax(420px, 2fr); gap: 20px;">
    <section class="panel" style="padding: 18px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
        <h2 style="margin-top: 0; font-size: 16px;">Create Access Key</h2>
        <div class="muted" style="font-size: 12px; margin-bottom: 14px;">Issue a new API credential with scoped capabilities for automation and CI/CD pipelines.</div>
        {creation_form}
    </section>
    <section class="panel" style="padding: 18px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
        <h2 style="margin-top: 0; font-size: 16px;">Access Keys Inventory</h2>
        <div class="muted" style="font-size: 12px; margin-bottom: 14px;">Existing cryptographic access credentials authorized for RPC endpoints.</div>
        {token_table}
    </section>
</div>"#,
        tokens.iter().filter(|t| !t.is_expired()).count(),
        creation_form = create_token_form(),
        token_table = token_table
    ))
}

fn render_with_created_token(
    workspace: &crate::core::workspace_queries::WorkspaceQueries,
    token: &crate::wallet::ApiToken,
    secret: &str,
) -> String {
    let tokens = workspace.list_api_tokens().unwrap_or_default();
    let token_table = render_token_table(&tokens);
    let breadcrumb = html::breadcrumb(&[("NeoNexus", "/"), ("API tokens", "")]);
    let head = html::page_head(
        "IAM Security Credentials & Access Keys",
        "Manage programmatic access keys, secret tokens, and least-privilege IAM permission boundaries.",
        r#"<a class="btn" href="/signer">KMS Custody</a>"#,
    );

    format!(
        r#"{breadcrumb}
{head}
<div class="panel" style="margin-bottom: 20px; padding: 20px; background: rgba(59, 209, 132, 0.08); border: 1px solid var(--jade); border-radius: 8px;">
    <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 8px;">
        <span class="badge running" style="font-size: 12px;">✓ Access Key Created Successfully</span>
        <span class="muted mono" style="font-size: 12px;">{id}</span>
    </div>
    <p style="font-size: 13px; color: #fff; margin: 6px 0 12px 0;">This is the only time your secret access token can be viewed or downloaded. You cannot recover it later.</p>
    <div style="display: flex; align-items: center; gap: 12px; background: rgba(0,0,0,0.4); padding: 12px 16px; border-radius: 6px; border: 1px solid var(--line); flex-wrap: wrap;">
        <code id="new-token-secret" class="mono" style="font-size: 14px; font-weight: 600; color: var(--jade); word-break: break-all;">{secret}</code>
        <button type="button" class="btn small primary" data-copy-target="new-token-secret">Copy Secret Key</button>
    </div>
    <div class="notice warn" style="margin-top: 12px; font-size: 12px;"><strong>Security Best Practice:</strong> Treat this token like a root password. Never commit secrets to public version control or insecure logs.</div>
</div>
<div class="grid" style="grid-template-columns: minmax(300px, 1fr) minmax(420px, 2fr); gap: 20px;">
    <section class="panel" style="padding: 18px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
        <h2 style="margin-top: 0; font-size: 16px;">Access Key Specifications</h2>
        {details}
    </section>
    <section class="panel" style="padding: 18px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
        <h2 style="margin-top: 0; font-size: 16px;">Access Keys Inventory</h2>
        {token_table}
    </section>
</div>"#,
        breadcrumb = breadcrumb,
        head = head,
        id = html::escape(&token.id.to_string()),
        secret = html::escape(secret),
        details = render_token_details(token),
        token_table = token_table
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
        r#"<dl class="token-meta" style="display: grid; grid-template-columns: 100px 1fr; gap: 8px; font-size: 13px;">
    <dt class="muted">Access Key ID</dt>
    <dd><code class="mono" style="color: var(--cyan);">{id_display}</code></dd>
    
    <dt class="muted">Token Name</dt>
    <dd><strong>{name}</strong></dd>
    
    <dt class="muted">Created</dt>
    <dd class="mono">{created_at}</dd>
    
    <dt class="muted">Expires</dt>
    <dd class="mono">{expires}</dd>
    
    <dt class="muted">Permissions</dt>
    <dd>{permissions_str}</dd>
    
    <dt class="muted">Status</dt>
    <dd><span class="badge {status_class}">{status}</span></dd>
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
    r#"<form method="post" action="/settings/api-tokens/create" class="filters" style="display: flex; flex-direction: column; gap: 14px;">
    <div class="form-group">
        <label for="token-name" style="font-weight: 600; font-size: 13px; display: block; margin-bottom: 4px;">Access Key Name</label>
        <input type="text" id="token-name" name="name" placeholder="e.g. CI-CD-Deployer or Hermes-Autonomic" required style="width: 100%;">
        <div class="muted" style="font-size: 11px; margin-top: 4px;">Descriptive identifier for IAM audit tracking in CloudTrail.</div>
    </div>
    
    <div class="form-group">
        <label style="font-weight: 600; font-size: 13px; display: block; margin-bottom: 6px;">IAM Permission Boundary Policies</label>
        <div style="display: flex; flex-direction: column; gap: 8px; background: rgba(0,0,0,0.2); padding: 10px; border-radius: 6px; border: 1px solid var(--line);">
            <label class="checkbox-label" style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer;">
                <input type="checkbox" name="read_fleet" value="true"> <code>read_fleet</code> — every node's configuration and state, read only
            </label>
            <label class="checkbox-label" style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer;">
                <input type="checkbox" name="read_readiness" value="true"> <code>read_readiness</code> — the readiness report, read only
            </label>
            <label class="checkbox-label" style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer;">
                <input type="checkbox" name="admin_all" value="true"> <code>admin_all</code> — everything, including starting and stopping nodes
            </label>
        </div>
        <div class="muted" style="font-size: 11px; margin-top: 4px;">Assign least-privilege permission grants required for programmatic caller.</div>
    </div>
    
    <button type="submit" class="btn primary" style="align-self: flex-start; margin-top: 4px;">+ Create Access Key</button>
</form>"#
        .to_string()
}

fn render_token_table(tokens: &[crate::wallet::ApiToken]) -> String {
    if tokens.is_empty() {
        return "<p class='muted' style='padding: 12px 0;'>No IAM access key credentials have been generated yet.</p>".to_string();
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
            let perms = token.permissions.iter().map(|p| format!(r#"<span class="badge">{}</span>"#, html::escape(&p.to_string()))).collect::<Vec<_>>().join(" ");

            let status_badge = if token.is_expired() {
                r#"<span class="badge danger">Expired</span>"#
            } else {
                r#"<span class="badge running">Active</span>"#
            };

            format!(
                r#"<tr>
    <td><strong>{name}</strong></td>
    <td><code class="mono" style="color: var(--cyan);">{id_display}</code></td>
    <td class="mono muted" style="font-size: 11px;">{created_at}</td>
    <td class="mono muted" style="font-size: 11px;">{expires}</td>
    <td>{perms}</td>
    <td>{status_badge}</td>
    <td>
        <form method="POST" action="/settings/api-tokens/{token_id}/delete" class="inline-form" data-confirm="Are you sure you want to permanently revoke API access key &#39;{name}&#39;?">
            <button type="submit" class="btn small danger">Revoke</button>
        </form>
    </td>
</tr>"#,
                name = name,
                id_display = id_display,
                created_at = created_at,
                expires = expires,
                perms = perms,
                status_badge = status_badge,
                token_id = token.id
            )
        })
        .collect();

    let rows_html = rows.join("\n");

    format!(
        r#"<div class="table-container">
<table class="dashboard-table">
    <thead>
        <tr>
            <th>Key Name</th>
            <th>Access Key ID</th>
            <th>Created</th>
            <th>Expires</th>
            <th>Policy Grants</th>
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
