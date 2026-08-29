//! Liveness endpoint for cloud probes and load balancers. Public by design:
//! it reports only process liveness, never workspace data.

use axum::{
    response::{IntoResponse, Response},
    Json,
};

pub async fn healthz() -> Response {
    Json(serde_json::json!({
        "status": "ok",
        "application": "NeoNexus",
        "version": env!("CARGO_PKG_VERSION"),
    }))
    .into_response()
}
