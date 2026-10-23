//! Readiness probe for an orchestrator.
//!
//! `/healthz` answers "is this process alive". `/ready` answers "should work
//! come here": it reports `503` while the workspace is blocked, so a load
//! balancer or a Kubernetes service stops sending traffic to an instance that
//! cannot serve it, and starts again once the operator has fixed what is
//! blocking.
//!
//! The probe is public, like `/healthz` and `/api/public/status`, and it says
//! only what those already publish: a verdict and counts. Which node is
//! blocked, and where its configuration lives, stay behind the operator
//! session and `/api/readiness`.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use log::error;

use super::{api, WebState};

/// `200` when the workspace can serve, `503` when it is blocked or cannot be
/// judged.
pub async fn ready(State(state): State<WebState>) -> Response {
    match api::fleet_diagnostics(&state) {
        Ok(diagnostics) => {
            let probe = api::readiness::probe(&diagnostics);
            let status = if probe.status == "blocked" {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::OK
            };
            (status, Json(probe)).into_response()
        }
        Err(error) => {
            // A workspace that cannot be read is not a workspace that can serve.
            error!("NeoNexus readiness probe failed: {error:#}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "status": "unknown" })),
            )
                .into_response()
        }
    }
}
