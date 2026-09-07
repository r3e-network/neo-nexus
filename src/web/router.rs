//! Route table and browser authentication boundary. Operator pages require a
//! session, and every state-changing operator request must also prove the exact
//! browser origin. The caller-facing signer relay keeps its bearer boundary.

use axum::{
    extract::{DefaultBodyLimit, Request, State},
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

use super::{api, control, health, pages, public_api, signer_api, signer_control, WebState};
use crate::signer_client::MAX_REQUEST_BODY_BYTES;

pub fn build_router(state: WebState) -> Router {
    // The public relay accepts raw bytes so workload signatures remain valid.
    // Bound those bytes before allocation and keep this layer off unrelated
    // browser and API routes.
    let signer_relay = Router::new()
        .route(
            "/signer/api/v1/sign/transaction",
            post(signer_api::sign_transaction),
        )
        .route(
            "/signer/api/v1/sign/consensus",
            post(signer_api::sign_consensus),
        )
        .route(
            "/signer/api/v1/sign/eip191-fulfillment",
            post(signer_api::sign_eip191_fulfillment),
        )
        .route("/signer/api/v1/sign/raw", post(signer_api::sign_raw))
        .route("/signer/api/v1/keys/{id}", get(signer_api::key_info))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES))
        // The outer admission layer runs before request extraction, so slow or
        // incomplete bodies consume the same bounded slots as active signer
        // calls. The timeout bounds header/body reading as well as dispatch.
        .layer(middleware::from_fn_with_state(
            state.clone(),
            signer_api::admit,
        ))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(15),
        ));

    let public = Router::new()
        .route(
            "/login",
            get(pages::login::login_page).post(pages::login::login_submit),
        )
        .route("/healthz", get(health::healthz))
        .route("/api/public/status", get(public_api::status))
        // The signing API is deliberately outside the session layer, which is
        // not the same as outside authentication: a caller is a program holding
        // a bearer token or workload identity, and the custody service checks
        // that proof, its origin binding and its key grant itself — this
        // workbench forwards the credential and decides nothing. Wrapping these paths in
        // `require_session` would answer `401` from a layer that has never heard
        // of the caller, and would keep the attempt out of the service's audit
        // trail.
        .merge(signer_relay)
        .route("/public-metrics", get(api::metrics_prometheus));

    let protected = Router::new()
        .route("/", get(pages::home::home))
        .route("/nodes", get(pages::nodes::node_list))
        .route(
            "/nodes/new",
            get(pages::node_editor::new_form).post(pages::node_editor::create),
        )
        .route("/nodes/{id}", get(pages::nodes::node_detail))
        .route(
            "/nodes/{id}/edit",
            get(pages::node_editor::edit_form).post(pages::node_editor::update),
        )
        .route(
            "/nodes/{id}/delete",
            get(pages::nodes::delete_form).post(pages::nodes::delete),
        )
        .route("/nodes/{id}/start", post(control::node_start))
        .route("/nodes/{id}/stop", post(control::node_stop))
        .route("/nodes/{id}/restart", post(control::node_restart))
        .route(
            "/nodes/{id}/signer",
            post(pages::nodes::save_signer_binding),
        )
        .route("/monitor", get(pages::monitor::monitor))
        .route("/logs", get(pages::logs::logs))
        .route("/operations", get(pages::operations::operations))
        .route("/events", get(pages::events::events))
        .route("/alerts", get(pages::alerts::alerts))
        .route("/alerts/routing", post(control::save_alert_routing))
        .route("/federation", get(pages::federation::federation))
        .route("/federation/{id}/toggle", post(pages::federation::toggle))
        .route("/federation/{id}/probes", get(pages::federation::probes))
        .route("/roles", get(pages::roles::roles))
        .route("/nodes/{id}/role", post(pages::roles::apply_role))
        .route("/config", get(pages::config::config))
        .route("/config/export", post(pages::config::export_all))
        .route("/plugins", get(pages::plugins::plugins))
        .route("/plugins/{id}/toggle", post(pages::plugins::toggle))
        .route("/runtimes", get(pages::runtimes::runtimes))
        .route("/runtimes/install", post(pages::runtimes::install))
        .route("/snapshots", get(pages::snapshots::snapshots))
        .route(
            "/snapshots/{snapshot_id}/apply/{node_id}",
            post(control::apply_snapshot),
        )
        .route("/wallets", get(pages::wallets::wallets))
        .route(
            "/backup",
            get(pages::backup::backup_page).post(control::handle_backup_export),
        )
        .route("/signer", get(pages::signer::signer))
        .route("/signer/keys/generate", post(signer_control::generate))
        .route("/signer/keys/{id}", get(pages::signer::key_detail))
        .route(
            "/signer/keys/{id}/delete",
            get(pages::signer::delete_form).post(signer_control::delete_key),
        )
        .route(
            "/signer/keys/{id}/state",
            post(signer_control::set_key_state),
        )
        .route(
            "/signer/keys/{id}/policy",
            post(signer_control::save_policy),
        )
        .route("/signer/callers", post(signer_control::create_caller))
        .route(
            "/signer/callers/workload",
            post(signer_control::create_workload_caller),
        )
        .route(
            "/signer/callers/{id}/rotate",
            get(pages::signer::caller_rotate_form).post(signer_control::rotate_caller),
        )
        .route(
            "/signer/callers/{id}/state",
            post(signer_control::set_caller_state),
        )
        .route(
            "/signer/callers/{id}/delete",
            get(pages::signer::caller_delete_form).post(signer_control::delete_caller),
        )
        .route("/metrics", get(pages::metrics_page::metrics))
        .route("/settings", get(pages::settings::settings))
        .route("/settings/density", post(control::save_density))
        .route("/settings/watchdog", post(control::save_watchdog))
        .route(
            "/settings/rpc-health",
            post(control::save_rpc_health_monitor),
        )
        .route(
            "/settings/federation",
            post(control::save_federation_monitor),
        )
        .route("/logout", post(logout))
        .route("/api/fleet", get(api::fleet))
        .route("/api/readiness", get(api::readiness))
        .route("/api/metrics-prometheus", get(api::metrics_prometheus))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_session,
        ));

    public
        .merge(protected)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            super::security_headers::apply,
        ))
        .with_state(state)
}

async fn require_session(State(state): State<WebState>, request: Request, next: Next) -> Response {
    let session_id = session_from_cookie(request.headers().get(header::COOKIE));
    let is_api = request.uri().path().starts_with("/api/");

    // Check if this is the public-metrics endpoint with optional token
    let path_is_public_metrics = request.uri().path() == "/public-metrics";

    // If it's public-metrics, allow either no auth (completely open) or token-based auth
    if path_is_public_metrics {
        return next.run(request).await;
    }

    if state.auth.session_is_valid(session_id) {
        if (is_unsafe_method(request.method()) || path_is_public_metrics)
            && !state
                .web_security()
                .allows_unsafe_request(request.headers())
        {
            return (
                StatusCode::FORBIDDEN,
                "state-changing requests require the configured web Origin or Referer",
            )
                .into_response();
        }
        return next.run(request).await;
    }
    if is_api {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            r#"{"error":"authentication required"}"#,
        )
            .into_response()
    } else {
        Redirect::to("/login").into_response()
    }
}

fn is_unsafe_method(method: &Method) -> bool {
    method != Method::GET && method != Method::HEAD && method != Method::OPTIONS
}

fn session_from_cookie(cookie_header: Option<&header::HeaderValue>) -> Option<&str> {
    let cookies = cookie_header?.to_str().ok()?;
    cookies.split(';').find_map(|pair| {
        let (name, value) = pair.trim().split_once('=')?;
        (name == super::auth::SESSION_COOKIE).then_some(value.trim())
    })
}

async fn logout(State(state): State<WebState>, request: Request) -> Response {
    let session_id = session_from_cookie(request.headers().get(header::COOKIE));
    state.auth.drop_session(session_id);
    (
        [
            (header::SET_COOKIE, state.clear_session_cookie()),
            (header::LOCATION, "/login".to_string()),
        ],
        StatusCode::SEE_OTHER,
    )
        .into_response()
}
