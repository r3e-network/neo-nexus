//! Permission-based authorization for the JSON API surface.
//!
//! The router authenticates each request once (browser session or API bearer
//! token) and tags it with an [`AuthIdentity`]. This module turns that identity
//! into a per-endpoint permission decision, keeping token scoping and route
//! wiring in one place instead of re-reading credentials on every handler.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::wallet::{ApiToken, TokenPermission};

/// Identity established by the outer authentication boundary and attached to the
/// request so downstream authorization can decide without re-reading headers.
#[derive(Clone, Debug)]
pub enum AuthIdentity {
    /// A signed-in browser operator, carrying full operator authority.
    Session,
    /// A verified API bearer token, limited to its granted permissions.
    Token(Box<ApiToken>),
}

/// Authorization middleware that enforces the permission a specific `/api/*`
/// route requires.
///
/// Authentication already happened in the router's session/bearer boundary,
/// which tagged the request with an [`AuthIdentity`]. A browser session carries
/// full operator authority and passes unconditionally; an API bearer token must
/// hold the required permission, with `AdminAll` implying every other grant.
pub async fn require_permission(
    request: Request,
    next: Next,
    required: api_permissions::RequiredPermission,
) -> Response {
    let identity = request.extensions().get::<AuthIdentity>().cloned();
    match identity {
        Some(AuthIdentity::Session) => next.run(request).await,
        Some(AuthIdentity::Token(token)) => {
            if token.has_permission(&required.permission()) {
                next.run(request).await
            } else {
                (
                    StatusCode::FORBIDDEN,
                    "Insufficient permissions for this endpoint",
                )
                    .into_response()
            }
        }
        None => (
            StatusCode::UNAUTHORIZED,
            r#"{"error":"authentication required"}"#,
        )
            .into_response(),
    }
}

/// Permission requirements for different API endpoints.
pub mod api_permissions {
    use super::TokenPermission;

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

    impl RequiredPermission {
        /// The token permission a caller must hold to satisfy this requirement.
        pub(crate) fn permission(self) -> TokenPermission {
            match self {
                RequiredPermission::ReadFleet => TokenPermission::ReadFleet,
                RequiredPermission::ReadReadiness => TokenPermission::ReadReadiness,
                RequiredPermission::AdminAll => TokenPermission::AdminAll,
            }
        }
    }
}
