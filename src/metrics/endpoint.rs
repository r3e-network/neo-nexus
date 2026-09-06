/// HTTP handler for /metrics endpoint exposing Prometheus-format metrics

use axum::{
    extract::State,
    response::{Response,ContentType},
    http::{Request,StatusCode} ,
};
use prometheus::Encoder;

use crate::metrics::prometheus_registry;

/// Metrics state held in application state
#[derive(Clone)]
pub struct MetricsState {
    /// Enable/disable metrics collection (feature flag)
    pub enabled: bool,
}

/// Handle GET /metrics requests
pub async fn handle_metrics<State>(
    _request: Request<axum::body::Body>,
) -> Result<Response<axum::body::Body>, StatusCode> {
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = vec![];
    
    // Encode all registered metrics
    if let Err(e) = encoder.encode(&prometheus_registry::get_registry().gather(), &mut buffer) {
        eprintln!("Failed to encode metrics: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    // Convert buffer to string and create response
    let output = String::from_utf8(buffer).unwrap_or_default();
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(ContentType::html())
        .body(axum::body::Body::from(output))
        .unwrap())
}
