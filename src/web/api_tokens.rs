//! Handler functions for API requests requiring permission-based authorization.
//!
//! These handlers wrap existing API routes and enforce fine-grained permissions
//! based on the authenticated API token's granted permissions.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::web::WebState;

/// Middleware layer that enforces API token permissions on /api/* routes.
pub async fn require_permission(
    State(state): State<WebState>,
    request: Request,
    next: Next,
    required_permission: api_permissions::RequiredPermission,
) -> Result<Response, (StatusCode, String)> {
    // Extract Bearer token from Authorization header
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .filter(|s| s.starts_with("Bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "Authorization header missing or invalid".to_string(),
            )
        })?;

    let token_secret = &auth_header["Bearer ".len()..];

    // Verify token and get its metadata
    let maybe_token = state
        .repository
        .verify_token_secret(token_secret)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Token verification error: {}", e),
            )
        })?;

    let token = maybe_token.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "Invalid or expired API token".to_string(),
        )
    })?;

    // Check if token has required permission
    let has_access = match required_permission {
        api_permissions::RequiredPermission::ReadFleet => {
            token.has_permission(&crate::wallet::TokenPermission::ReadFleet)
        }
        api_permissions::RequiredPermission::ReadReadiness => {
            token.has_permission(&crate::wallet::TokenPermission::ReadReadiness)
        }
        api_permissions::RequiredPermission::AdminAll => {
            token.has_permission(&crate::wallet::TokenPermission::AdminAll)
        }
    };

    if has_access {
        Ok(next.run(request).await)
    } else {
        Err((
            StatusCode::FORBIDDEN,
            "Insufficient permissions for this endpoint".to_string(),
        ))
    }
}

/// Permission requirements for different API endpoints.
pub mod api_permissions {
    /// Required permission level for an API endpoint.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RequiredPermission {
        /// Access to fleet data (/api/fleet, GET operations)
        ReadFleet,
        /// Access to readiness status (/api/readiness, GET operations)
        ReadReadiness,
        /// Full admin access (all read + write operations)
        AdminAll,
    }
}
