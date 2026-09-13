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

impl AuthIdentity {
    /// The single node this identity is confined to, if it is confined at all.
    ///
    /// A guest agent token is issued to one instance and speaks only for that
    /// instance, the way a cloud instance profile does. A token that also holds
    /// a fleet-wide grant is not confined — the operator asked for something
    /// broader and got it — so confinement is the absence of any other grant,
    /// not merely the presence of this one.
    pub fn confined_to_node(&self) -> Option<&str> {
        let Self::Token(token) = self else {
            return None;
        };
        let mut confined: Option<&str> = None;
        for permission in &token.permissions {
            match permission {
                TokenPermission::HermesAgent(node_id) => {
                    // Two different instances on one token is not a confinement
                    // this model can express, so it is not treated as one.
                    if confined.is_some_and(|seen| seen != node_id.as_str()) {
                        return None;
                    }
                    confined = Some(node_id.as_str());
                }
                TokenPermission::ReadFleet
                | TokenPermission::ReadReadiness
                | TokenPermission::AdminAll => return None,
            }
        }
        confined
    }

    /// Whether this identity may address `node_id` at all.
    pub fn may_access_node(&self, node_id: &str) -> bool {
        self.confined_to_node()
            .is_none_or(|confined| confined == node_id)
    }
}

/// Whether a request path lies inside one instance's own namespace.
///
/// This is the whole point of the confinement: a token issued to one instance
/// must not reach a route that addresses another, and must not reach the
/// fleet-wide routes at all. Deciding from the path rather than from a list of
/// endpoints means a route added later is confined by default, instead of being
/// open to every guest agent until someone remembers to gate it.
pub(crate) fn path_is_within_node_namespace(path: &str, node_id: &str) -> bool {
    let mut segments = path.split('/').filter(|segment| !segment.is_empty());
    if segments.next() != Some("api") || segments.next() != Some("nodes") {
        return false;
    }
    segments
        .next()
        .is_some_and(|addressed| crate::web::html::percent_decode(addressed) == node_id)
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
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum RequiredPermission {
        /// Access to fleet data (/api/fleet, GET operations)
        ReadFleet,
        /// Access to readiness status (/api/readiness, GET operations)
        ReadReadiness,
        /// Access to node-specific metrics and logs (/api/nodes/{id}/metrics)
        ReadNodeMetrics,
        /// Full admin access (all read + write operations)
        AdminAll,
        /// Scoped Hermes Agent access to a specific node instance
        HermesAgent(String),
    }

    impl RequiredPermission {
        /// The token permission a caller must hold to satisfy this requirement.
        pub(crate) fn permission(&self) -> TokenPermission {
            match self {
                RequiredPermission::ReadFleet => TokenPermission::ReadFleet,
                RequiredPermission::ReadReadiness => TokenPermission::ReadReadiness,
                RequiredPermission::ReadNodeMetrics => TokenPermission::ReadFleet, // Reuse fleet read permission
                RequiredPermission::AdminAll => TokenPermission::AdminAll,
                RequiredPermission::HermesAgent(node_id) => {
                    TokenPermission::HermesAgent(node_id.clone())
                }
            }
        }
    }
}
