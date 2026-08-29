//! Route table and the authentication boundary. Everything except the login
//! page and `/healthz` requires a valid session cookie: pages redirect to
//! `/login`, API routes answer `401` so the polling script can re-authenticate.

use axum::{
    extract::{Request, State},
    http::header,
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};

use super::{api, control, health, pages, WebState};

pub fn build_router(state: WebState) -> Router {
    let public = Router::new()
        .route(
            "/login",
            get(pages::login::login_page).post(pages::login::login_submit),
        )
        .route("/healthz", get(health::healthz));

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
        .route("/monitor", get(pages::monitor::monitor))
        .route("/logs", get(pages::logs::logs))
        .route("/operations", get(pages::operations::operations))
        .route("/alerts", get(pages::alerts::alerts))
        .route("/alerts/routing", post(control::save_alert_routing))
        .route("/federation", get(pages::federation::federation))
        .route("/federation/{id}/toggle", post(pages::federation::toggle))
        .route("/federation/{id}/probes", get(pages::federation::probes))
        .route("/roles", get(pages::roles::roles))
        .route("/config", get(pages::config::config))
        .route("/config/export", post(pages::config::export_all))
        .route("/plugins", get(pages::plugins::plugins))
        .route("/plugins/{id}/toggle", post(pages::plugins::toggle))
        .route("/runtimes", get(pages::runtimes::runtimes))
        .route("/runtimes/install", post(pages::runtimes::install))
        .route("/snapshots", get(pages::snapshots::snapshots))
        .route("/wallets", get(pages::wallets::wallets))
        .route("/metrics", get(pages::metrics_page::metrics))
        .route("/settings", get(pages::settings::settings))
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

    public.merge(protected).with_state(state)
}

async fn require_session(State(state): State<WebState>, request: Request, next: Next) -> Response {
    let session_id = session_from_cookie(request.headers().get(header::COOKIE));
    let is_api = request.uri().path().starts_with("/api/");
    if state.auth.session_is_valid(session_id) {
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
    Redirect::to("/login").into_response()
}
