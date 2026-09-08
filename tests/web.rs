//! End-to-end web workbench coverage: a real server on an ephemeral port, a
//! real workspace database, and plain HTTP through the library's own `ureq`
//! dependency. The suite pins the auth boundary, the JSON API, the page render,
//! the lifecycle control path, and key custody — the same pipeline the CLI uses.

use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use axum::serve;
use neo_nexus::{
    core::operations::RuntimeEventFilter,
    repository::Repository,
    signer_client::{SignerClient, SignerConfig},
    types::{Network, NewNode, NodeType, StorageEngine},
    watchdog::RestartPolicy,
    web::{
        auth::{AuthStore, WebSecurity},
        html, nav,
        router::build_router,
        Custody, WebState,
    },
};
use ureq::AgentBuilder;

const TOKEN: &str = "web-suite-token-7f019eb552e74a55b96a8e7817d79e93";

struct Server {
    base_url: String,
    db_path: PathBuf,
    /// The state the router serves with, kept so a supervised test can hand the
    /// engine the *same* supervisor rather than a second one.
    state: WebState,
    /// The supervision engine, when the test asked for one. Production always
    /// runs it; most tests do not need it and would only pay for the probes.
    _engine: Option<neo_nexus::supervision::Engine>,
    // The runtime owns the accept loop; dropping it stops the server, so it has
    // to outlive every request the test makes. Declared before `_home` so the
    // server is shut down before the temp workspace disappears.
    _runtime: tokio::runtime::Runtime,
    // Keep the tempdir alive for the whole test.
    _home: tempfile::TempDir,
}

fn spawn_server() -> Server {
    // Pinned rather than left to `WebState::new`'s read of the environment: a
    // suite whose custody surface depends on which variables happen to be set on
    // the machine running it is a suite that passes differently in two places.
    spawn_server_custody(Custody::unconfigured())
}

/// The same workspace with custody decided by the test. The value has to be in
/// place before `build_router` consumes the state, since that clone — not this
/// one — is what answers the requests.
fn spawn_server_custody(custody: Custody) -> Server {
    spawn_server_custody_with_relay_limit(custody, None)
}

fn spawn_server_custody_with_relay_limit(custody: Custody, relay_limit: Option<usize>) -> Server {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let home = tempfile::tempdir().expect("temp workspace dir");
    let db_path = home.path().join("neonexus.db");
    Repository::open(&db_path).expect("workspace database");
    let state = WebState::new(
        Repository::open(&db_path).expect("workspace repository"),
        home.path().to_path_buf(),
        AuthStore::from_token(TOKEN).expect("strong test token"),
        WebSecurity::loopback_http(),
    )
    .expect("valid state configuration")
    .with_custody(custody);
    let state = match relay_limit {
        Some(limit) => state.with_signer_relay_limit(limit),
        None => state,
    };
    // `build_router` consumes its state, so hand it a clone and keep the
    // original: WebState is shared by design and cheap to clone.
    let router_state = state.clone();
    let address = runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("ephemeral bind");
        let address = listener.local_addr().expect("bound address");
        tokio::spawn(async move {
            serve(
                listener,
                build_router(router_state)
                    .into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .expect("server task");
        });
        address
    });
    Server {
        base_url: format!("http://{address}"),
        db_path,
        state,
        _engine: None,
        _runtime: runtime,
        _home: home,
    }
}

/// The same server with the supervision engine running, so a test can watch the
/// watchdog do what the Settings page promises it does.
fn spawn_supervised_server() -> Server {
    let mut server = spawn_server();
    // Share the router's state exactly as `serve()` does. A second WebState
    // would mean a second supervisor, and the engine would then treat every
    // browser-started node as an unmanaged outsider.
    server._engine = Some(
        neo_nexus::supervision::Engine::start(server.state.engine_state())
            .expect("supervision engine starts"),
    );
    server
}

/// A command that exits non-zero immediately, so the watchdog has something
/// real to notice.
fn crashing_command() -> (PathBuf, Vec<String>) {
    if cfg!(windows) {
        (
            PathBuf::from(r"C:\Windows\System32\cmd.exe"),
            vec!["/c".to_string(), "exit 3".to_string()],
        )
    } else {
        (
            PathBuf::from("/bin/sh"),
            vec!["-c".to_string(), "exit 3".to_string()],
        )
    }
}

/// Poll `check` until it says yes or the deadline passes.
fn wait_until(timeout: Duration, mut check: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if check() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

fn agent() -> ureq::Agent {
    AgentBuilder::new()
        .redirects(0)
        .timeout(Duration::from_secs(10))
        .build()
}

/// ureq reports every status `>= 400` as `Error::Status`, but the auth boundary
/// is *meant* to answer 401 — so fold both arms back into a response and let
/// the assertions test the status code itself. Only transport failures abort.
fn into_response(result: Result<ureq::Response, ureq::Error>) -> ureq::Response {
    let response = match result {
        Ok(response) => Some(response),
        Err(ureq::Error::Status(_, response)) => Some(response),
        Err(_) => None,
    };
    response.expect("request reaches the workbench server")
}

/// Parse a response body as JSON. Asserting on the parsed document keeps the
/// suite independent of how serde_json chooses to space its output.
fn json_body(response: ureq::Response) -> serde_json::Value {
    let text = response.into_string().expect("utf-8 response body");
    let parsed = serde_json::from_str(&text);
    assert!(parsed.is_ok(), "response body is not JSON: {text}");
    parsed.expect("JSON validity checked above")
}

fn post_form(agent: &ureq::Agent, url: &str, body: &str) -> ureq::Response {
    into_response(
        agent
            .post(url)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string(body),
    )
}

/// The same post, signed in. `send_string` already resolves to a response, so
/// there is no `.call()` step to chain.
fn post_form_as(agent: &ureq::Agent, session: &str, url: &str, body: &str) -> ureq::Response {
    let origin = request_origin(url);
    into_response(
        agent
            .post(url)
            .set("cookie", session)
            .set("origin", &origin)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string(body),
    )
}

fn request_origin(url: &str) -> String {
    let parsed = url::Url::parse(url).expect("absolute test URL");
    let host = parsed.host_str().expect("test URL host");
    let authority = parsed
        .port()
        .map_or_else(|| host.to_string(), |port| format!("{host}:{port}"));
    format!("{}://{authority}", parsed.scheme())
}

fn cookie_value(response: &ureq::Response) -> Option<String> {
    response
        .header("set-cookie")?
        .split(';')
        .next()
        .map(str::to_string)
}

fn create_node(db_path: &Path, name: &str, rpc_port: u16) -> String {
    let repository = Repository::open(db_path).expect("reopen workspace");
    let node = repository
        .create_node(NewNode {
            name: name.to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: PathBuf::from("./neo-node"),
            args: Vec::new(),
            runtime_version: "v0.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port,
            p2p_port: rpc_port + 1,
            ws_port: None,
        })
        .expect("node creation");
    node.id
}

#[test]
fn healthz_is_public() {
    let server = spawn_server();
    let response = into_response(agent().get(&format!("{}/healthz", server.base_url)).call());
    assert_eq!(response.status(), 200);
    assert_eq!(response.header("cache-control"), Some("no-store"));
    assert_eq!(response.header("x-content-type-options"), Some("nosniff"));
    assert_eq!(response.header("x-frame-options"), Some("DENY"));
    assert_eq!(response.header("referrer-policy"), Some("same-origin"));
    assert_eq!(
        response.header("cross-origin-opener-policy"),
        Some("same-origin")
    );
    assert_eq!(
        response.header("cross-origin-resource-policy"),
        Some("same-origin")
    );
    let content_security_policy = response
        .header("content-security-policy")
        .expect("content security policy");
    assert!(content_security_policy.contains("default-src 'none'"));
    assert!(content_security_policy.contains("script-src 'sha256-"));
    assert!(content_security_policy.contains("style-src 'sha256-"));
    assert!(!content_security_policy.contains("'unsafe-inline'"));
    assert_eq!(response.header("strict-transport-security"), None);
    let body = json_body(response);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["application"], "NeoNexus");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn federation_api_exposes_only_aggregate_status() {
    let server = spawn_server();
    create_node(&server.db_path, "Private node identity", 19332);
    let http = agent();

    let status = json_body(into_response(
        http.get(&format!("{}/api/public/status", server.base_url))
            .call(),
    ));
    assert_eq!(status["status"]["totalNodes"], 1);
    assert_eq!(status["status"]["runningNodes"], 0);
    assert_eq!(status["status"]["syncingNodes"], 0);
    assert_eq!(status["status"]["errorNodes"], 0);
    assert!(status["status"]["totalBlocks"].is_null());
    assert!(status["status"]["totalPeers"].is_null());
    assert!(status["status"]["timestamp"].as_u64().is_some());

    for path in [
        "/api/public/nodes",
        "/api/public/nodes/not-public",
        "/api/public/nodes/not-public/health",
        "/api/public/metrics/system",
        "/api/public/metrics/nodes",
    ] {
        let response = into_response(http.get(&format!("{}{path}", server.base_url)).call());
        assert_eq!(response.status(), 404, "{path} must not expose inventory");
    }
}

#[test]
fn pages_redirect_and_api_rejects_without_a_session() {
    let server = spawn_server();
    let http = agent();

    let home = into_response(http.get(&format!("{}/", server.base_url)).call());
    assert_eq!(home.status(), 303);
    assert_eq!(home.header("location"), Some("/login"));

    let fleet = into_response(http.get(&format!("{}/api/fleet", server.base_url)).call());
    assert_eq!(fleet.status(), 401);
}

#[test]
fn login_rejects_wrong_token_and_issues_a_session_for_the_right_one() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;

    let rejected = post_form(&http, &format!("{base}/login"), "token=not-the-token");
    assert_eq!(rejected.status(), 303);
    assert_eq!(rejected.header("location"), Some("/login?error=1"));

    let accepted = post_form(&http, &format!("{base}/login"), &format!("token={TOKEN}"));
    assert_eq!(accepted.status(), 303);
    assert_eq!(accepted.header("location"), Some("/"));
    let set_cookie = accepted
        .header("set-cookie")
        .expect("session cookie header");
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Strict"));
    assert!(
        !set_cookie.contains("Secure"),
        "loopback HTTP cookie: {set_cookie}"
    );
    let session = cookie_value(&accepted).expect("session cookie set");
    assert!(session.starts_with("neonexus_session="));

    let home = into_response(http.get(base).set("cookie", &session).call());
    assert_eq!(home.status(), 200);
    assert!(home.into_string().unwrap().contains("Fleet overview"));
}

#[test]
fn login_attempts_are_throttled_per_socket_peer() {
    let server = spawn_server();
    let http = agent();
    let url = format!("{}/login", server.base_url);

    for attempt in 1..=5 {
        let response = post_form(&http, &url, "token=wrong");
        if attempt < 5 {
            assert_eq!(response.status(), 303, "attempt {attempt}");
        } else {
            assert_eq!(response.status(), 429, "attempt {attempt}");
            assert_eq!(response.header("retry-after"), Some("5"));
        }
    }
    let still_blocked = post_form(&http, &url, &format!("token={TOKEN}"));
    assert_eq!(still_blocked.status(), 429);
    assert!(still_blocked.header("set-cookie").is_none());
}

#[test]
fn protected_mutations_require_exact_origin_or_matching_referer() {
    let server = spawn_server();
    let http = agent();
    let session = signed_in(&http, &server.base_url);
    let node_id = create_node(&server.db_path, "csrf-target", 23332);
    let url = format!("{}/nodes/{node_id}/delete", server.base_url);

    let missing = into_response(http.post(&url).set("cookie", &session).call());
    assert_eq!(missing.status(), 403);
    let sibling = into_response(
        http.post(&url)
            .set("cookie", &session)
            .set("origin", "http://sibling.localhost")
            .call(),
    );
    assert_eq!(sibling.status(), 403);
    assert!(Repository::open(&server.db_path)
        .unwrap()
        .list_nodes()
        .unwrap()
        .iter()
        .any(|node| node.id == node_id));

    let accepted = into_response(
        http.post(&url)
            .set("cookie", &session)
            .set("referer", &format!("{}/nodes/{node_id}", server.base_url))
            .call(),
    );
    assert_eq!(accepted.status(), 303);
    assert!(Repository::open(&server.db_path)
        .unwrap()
        .list_nodes()
        .unwrap()
        .iter()
        .all(|node| node.id != node_id));
}

#[test]
fn public_metrics_route_accessible_without_authentication() {
    let server = spawn_server();
    let http = agent();

    // /public-metrics should be accessible without any authentication
    let response = into_response(
        http.get(&format!("{}/public-metrics", server.base_url))
            .call(),
    );
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.header("content-type"),
        Some("text/plain; version=0.0.4")
    );
    let body = response.into_string().expect("utf-8 response body");
    assert!(
        body.contains("# HELP"),
        "Prometheus metrics must start with help comments"
    );
    assert!(
        body.contains("# TYPE"),
        "Prometheus metrics must contain type declarations"
    );
}

#[test]
fn api_metrics_prometheus_requires_session_authentication() {
    let server = spawn_server();
    let http = agent();

    // /api/metrics-prometheus should require session cookie
    let response = into_response(
        http.get(&format!("{}/api/metrics-prometheus", server.base_url))
            .call(),
    );
    assert_eq!(response.status(), 401);
}

#[test]
fn fleet_api_lists_created_nodes_and_control_persists_state() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;

    let login = post_form(&http, &format!("{base}/login"), &format!("token={TOKEN}"));
    let session = cookie_value(&login).expect("session cookie set");

    let node_id = create_node(&server.db_path, "web-suite-node", 21332);

    let fleet = into_response(
        http.get(&format!("{base}/api/fleet"))
            .set("cookie", &session)
            .call(),
    );
    assert_eq!(fleet.status(), 200);
    let nodes = json_body(fleet)["nodes"]
        .as_array()
        .expect("nodes array")
        .clone();
    let row = nodes
        .iter()
        .find(|node| node["name"] == "web-suite-node")
        .expect("created node is listed");
    assert_eq!(row["status"], "Stopped");
    assert_eq!(row["rpc_port"], 21332);

    let stop = into_response(
        http.post(&format!("{base}/nodes/{node_id}/stop"))
            .set("cookie", &session)
            .set("origin", base)
            .call(),
    );
    assert_eq!(stop.status(), 303);
    let location = stop.header("location").expect("redirect back to node");
    assert!(location.starts_with(&format!("/nodes/{node_id}?flash=")));
    assert!(location.contains("was%20not%20running"));

    let repository = Repository::open(&server.db_path).expect("reopen workspace");
    let persisted = repository
        .list_nodes()
        .expect("nodes")
        .into_iter()
        .find(|node| node.id == node_id)
        .expect("created node");
    assert_eq!(
        persisted.status,
        neo_nexus::types::NodeStatus::Stopped,
        "stop must persist Stopped even when nothing was running"
    );
}

/// The Metrics page tells operators to scrape this path, so the route has to
/// exist, serve text, and stay behind the same session boundary as the API.
#[test]
fn prometheus_exposition_is_served_behind_the_session() {
    let server = spawn_server();
    let http = agent();
    let path = format!("{}/api/metrics-prometheus", server.base_url);

    let anonymous = into_response(http.get(&path).call());
    assert_eq!(anonymous.status(), 401);

    let login = post_form(
        &http,
        &format!("{}/login", server.base_url),
        &format!("token={TOKEN}"),
    );
    let session = cookie_value(&login).expect("session cookie set");

    let scraped = into_response(http.get(&path).set("cookie", &session).call());
    assert_eq!(scraped.status(), 200);
    assert!(scraped
        .header("content-type")
        .unwrap_or_default()
        .starts_with("text/plain"));
    assert!(scraped
        .into_string()
        .expect("exposition body")
        .contains("neonexus_"));
}

/// Every sidebar destination is a protected page: anonymous requests are turned
/// away and signed-in requests render the workbench shell. The list comes from
/// the navigation table itself, so a new page is covered the moment it appears
/// in the sidebar rather than when someone remembers this test.
#[test]
fn every_sidebar_destination_is_protected_and_renders() {
    let server = spawn_server();
    let http = agent();
    let login = post_form(
        &http,
        &format!("{}/login", server.base_url),
        &format!("token={TOKEN}"),
    );
    let session = cookie_value(&login).expect("session cookie set");

    let keys = nav::keys();
    assert!(
        keys.len() >= 15,
        "expected the full workbench navigation, found {keys:?}"
    );
    for key in keys {
        let href = nav::href_for(key).expect("destination resolves");
        let url = format!("{}{}", server.base_url, href);

        let anonymous = into_response(http.get(&url).call());
        assert_eq!(
            anonymous.status(),
            303,
            "{key} at {href} must redirect when signed out"
        );

        let page = into_response(http.get(&url).set("cookie", &session).call());
        assert_eq!(
            page.status(),
            200,
            "{key} at {href} must render when signed in"
        );
        let body = page.into_string().expect("page body");
        assert!(
            body.contains("nav-item"),
            "{key} at {href} rendered without the workbench shell"
        );
    }
}

/// A policy form must persist what it was given, and refuse to store a value it
/// cannot parse rather than quietly saving zero.
#[test]
fn settings_form_persists_a_policy_and_rejects_unparseable_input() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let login = post_form(&http, &format!("{base}/login"), &format!("token={TOKEN}"));
    let session = cookie_value(&login).expect("session cookie set");

    let saved = post_form_as(
        &http,
        &session,
        &format!("{base}/settings/watchdog"),
        "enabled=Enabled&max_restart_attempts=7&base_delay_seconds=3&max_delay_seconds=90",
    );
    assert_eq!(saved.status(), 303);
    let location = saved.header("location").expect("redirect back to settings");
    assert!(
        location.contains("flash="),
        "outcome must reach the operator: {location}"
    );

    let repository = Repository::open(&server.db_path).expect("reopen workspace");
    let policy = repository.load_watchdog_policy().expect("watchdog policy");
    assert!(policy.enabled);
    assert_eq!(policy.max_restart_attempts, 7);
    assert_eq!(policy.base_delay, std::time::Duration::from_secs(3));
    assert_eq!(policy.max_delay, std::time::Duration::from_secs(90));

    let rejected = post_form_as(
        &http,
        &session,
        &format!("{base}/settings/watchdog"),
        "enabled=Enabled&max_restart_attempts=seven&base_delay_seconds=3&max_delay_seconds=90",
    );
    assert_eq!(rejected.status(), 303);
    let location = rejected
        .header("location")
        .expect("redirect back to settings");
    assert!(
        location.contains("not%20saved"),
        "a refused save must say so: {location}"
    );
    let unchanged = repository.load_watchdog_policy().expect("watchdog policy");
    assert_eq!(
        unchanged.max_restart_attempts, 7,
        "a rejected form must leave the stored policy alone"
    );
}

/// A blank webhook field means "keep what is stored". Echoing the redacted value
/// back into the database would destroy the real target.
#[test]
fn alert_routing_form_keeps_the_stored_webhook_when_the_field_is_blank() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let login = post_form(&http, &format!("{base}/login"), &format!("token={TOKEN}"));
    let session = cookie_value(&login).expect("session cookie set");

    let post = |body: &str| post_form_as(&http, &session, &format!("{base}/alerts/routing"), body);

    let stored = "https://hooks.example.test/token=SUPER-SECRET-VALUE";
    let first = post(&format!(
        "enabled=Enabled&provider=slack&min_severity=warning&webhook_url={}&timeout_seconds=5",
        html::urlencoding_lite(stored)
    ));
    assert_eq!(first.status(), 303);

    let repository = Repository::open(&server.db_path).expect("reopen workspace");
    let policy = repository
        .load_alert_routing_policy()
        .expect("routing policy");
    assert_eq!(policy.webhook_url.as_deref(), Some(stored));

    let second = post(
        "enabled=Enabled&provider=discord&min_severity=critical&webhook_url=&timeout_seconds=9",
    );
    assert_eq!(second.status(), 303);
    let after = repository
        .load_alert_routing_policy()
        .expect("routing policy");
    assert_eq!(
        after.webhook_url.as_deref(),
        Some(stored),
        "a blank field must not overwrite the stored target"
    );
    assert_eq!(after.provider, neo_nexus::alerts::AlertProvider::Discord);
    assert_eq!(after.timeout_seconds, 9);

    // The page must never put the secret back into the markup.
    let page = into_response(
        http.get(&format!("{base}/alerts"))
            .set("cookie", &session)
            .call(),
    );
    let body = page.into_string().expect("alerts page");
    assert!(
        !body.contains("SUPER-SECRET-VALUE"),
        "webhook token leaked to the browser"
    );
    assert!(
        body.contains("hooks.example.test"),
        "the host must stay visible so the operator can still recognise the hook"
    );
}

/// A control that cannot be honoured has to say so instead of half-applying it.
#[test]
fn plugin_toggle_refuses_an_identifier_that_is_not_a_plugin() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let login = post_form(&http, &format!("{base}/login"), &format!("token={TOKEN}"));
    let session = cookie_value(&login).expect("session cookie set");
    let node_id = create_node(&server.db_path, "toggle-node", 21442);

    let response = post_form_as(
        &http,
        &session,
        &format!("{base}/plugins/{node_id}/toggle"),
        "plugin=NoSuchPlugin",
    );
    assert_eq!(response.status(), 303);
    let location = response
        .header("location")
        .expect("redirect back to plugins");
    assert!(
        location.contains("not%20changed"),
        "a refused toggle must report it: {location}"
    );

    let repository = Repository::open(&server.db_path).expect("reopen workspace");
    assert!(
        repository
            .list_plugin_states(&node_id)
            .expect("plugin states")
            .is_empty(),
        "a rejected toggle must write nothing"
    );
}

/// A node form body. Deliberately spelled out rather than built from a draft,
/// so the test asserts what a browser would actually send.
fn node_form(name: &str, client: &str, rpc: &str, p2p: &str) -> String {
    format!(
        "name={name}&node_type={client}&network=mainnet&binary_path=%2Fopt%2Fneo%2Fnode\
&runtime_version=&storage_engine=rocksdb&args=&rpc_port={rpc}&p2p_port={p2p}&ws_port="
    )
}

fn signed_in(http: &ureq::Agent, base: &str) -> String {
    let login = post_form(http, &format!("{base}/login"), &format!("token={TOKEN}"));
    cookie_value(&login).expect("session cookie set")
}

/// The whole point of the 4.0 workbench: a node can be registered, corrected and
/// removed from a browser, with no access to the database.
#[test]
fn a_node_is_created_shown_edited_and_deleted_over_http() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);

    let form = into_response(
        http.get(&format!("{base}/nodes/new"))
            .set("cookie", &session)
            .call(),
    );
    assert_eq!(form.status(), 200);
    let markup = form.into_string().expect("editor markup");
    assert!(
        markup.contains("Add node"),
        "the editor should announce itself"
    );
    assert!(
        markup.contains("name=\"rpc_port\""),
        "ports are editable: {markup}"
    );

    let created = post_form_as(
        &http,
        &session,
        &format!("{base}/nodes/new"),
        &node_form("web-seed", "neo-rs", "31332", "31333"),
    );
    assert_eq!(created.status(), 303);
    let location = created.header("location").expect("redirect to the node");
    assert!(
        location.starts_with("/nodes/node-"),
        "expected a node page, got {location}"
    );
    assert!(
        location.contains("flash="),
        "the operator should be told it worked"
    );

    let repository = Repository::open(&server.db_path).expect("reopen workspace");
    let node = repository
        .list_nodes()
        .expect("nodes")
        .into_iter()
        .find(|node| node.name == "web-seed")
        .expect("the node should be stored");
    assert_eq!(node.rpc_port, 31332);
    assert_eq!(
        node.runtime_version, "latest",
        "a blank version means latest"
    );

    // It should be reachable from the list, and offer its own controls.
    let list = into_response(
        http.get(&format!("{base}/nodes"))
            .set("cookie", &session)
            .call(),
    );
    let list_body = list.into_string().expect("list body");
    assert!(
        list_body.contains("web-seed"),
        "the fleet list should show it"
    );
    assert!(list_body.contains("/edit"), "each row should offer editing");
    assert!(
        list_body.contains("/delete"),
        "each row should offer deletion"
    );

    let edited = post_form_as(
        &http,
        &session,
        &format!("{base}/nodes/{}/edit", node.id),
        &node_form("web-seed", "neo-rs", "31340", "31341"),
    );
    assert_eq!(edited.status(), 303);
    let moved = repository
        .list_nodes()
        .expect("nodes")
        .into_iter()
        .find(|stored| stored.id == node.id)
        .expect("still present");
    assert_eq!(moved.rpc_port, 31340, "the edit should persist");

    let confirm = into_response(
        http.get(&format!("{base}/nodes/{}/delete", node.id))
            .set("cookie", &session)
            .call(),
    );
    assert_eq!(confirm.status(), 200);
    assert!(
        confirm
            .into_string()
            .expect("confirm body")
            .contains("cannot be undone"),
        "deletion must warn before it acts"
    );

    let removed = post_form_as(
        &http,
        &session,
        &format!("{base}/nodes/{}/delete", node.id),
        "",
    );
    assert_eq!(removed.status(), 303);
    assert!(repository
        .list_nodes()
        .expect("nodes")
        .iter()
        .all(|stored| stored.id != node.id));

    // The journal should be able to tell the story afterwards, including the
    // deletion, which has no node row left to name it.
    let kinds = repository
        .list_events(RuntimeEventFilter::new(None, "", 200))
        .expect("events")
        .iter()
        .map(|event| event.kind.to_string())
        .collect::<Vec<_>>();
    assert!(
        kinds.iter().any(|kind| kind == "node-created"),
        "journal: {kinds:?}"
    );
    assert!(
        kinds.iter().any(|kind| kind == "node-updated"),
        "journal: {kinds:?}"
    );
    assert!(
        kinds.iter().any(|kind| kind == "node-deleted"),
        "journal: {kinds:?}"
    );
}

/// A rejected save must return the operator's own text and the reason beside the
/// field, not a blank form that throws their work away.
#[test]
fn a_rejected_save_keeps_the_operators_text_and_names_the_field() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);

    let response = post_form_as(
        &http,
        &session,
        &format!("{base}/nodes/new"),
        &node_form("typo-node", "neo-rs", "not-a-port", "31333"),
    );
    assert_eq!(
        response.status(),
        200,
        "a rejected form re-renders, not redirects"
    );
    let body = response.into_string().expect("form body");
    assert!(
        body.contains("value=\"typo-node\""),
        "the name should survive: {body}"
    );
    assert!(
        body.contains("not-a-port"),
        "the bad port should survive for correction"
    );
    assert!(
        body.contains("field needs attention") || body.contains("needs attention"),
        "the operator should be told something is wrong"
    );
    assert!(
        body.contains("is not a port number"),
        "and which field: {body}"
    );

    let repository = Repository::open(&server.db_path).expect("reopen workspace");
    assert!(
        repository.list_nodes().expect("nodes").is_empty(),
        "nothing may be stored"
    );
}

#[test]
fn a_duplicate_name_is_refused_over_http() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);
    create_node(&server.db_path, "taken-name", 32332);

    let response = post_form_as(
        &http,
        &session,
        &format!("{base}/nodes/new"),
        &node_form("TAKEN-NAME", "neo-rs", "33332", "33333"),
    );
    let body = response.into_string().expect("form body");
    assert!(
        body.contains("already used by another node"),
        "expected a clash message: {body}"
    );
}

/// The editor routes are not in the sidebar, so the navigation-driven sweep does
/// not reach them. They guard workspace changes all the same.
#[test]
fn editor_routes_require_a_session() {
    let server = spawn_server();
    let http = agent();
    let base = server.base_url.as_str();
    let node_id = create_node(&server.db_path, "guarded", 34332);

    for path in [
        "/nodes/new".to_string(),
        format!("/nodes/{node_id}/edit"),
        format!("/nodes/{node_id}/delete"),
    ] {
        let response = into_response(http.get(&format!("{base}{path}")).call());
        assert_eq!(
            response.status(),
            303,
            "{path} must redirect when signed out"
        );
    }
    for path in ["/nodes/new".to_string(), format!("/nodes/{node_id}/delete")] {
        let response = into_response(
            http.post(&format!("{base}{path}"))
                .set("content-type", "application/x-www-form-urlencoded")
                .send_string(&node_form("any", "neo-rs", "35332", "35333")),
        );
        assert_eq!(
            response.status(),
            303,
            "{path} must redirect when signed out"
        );
    }
}

/// A command that stays running long enough to observe, on every platform the
/// suite runs on.
fn long_running_command() -> (PathBuf, Vec<String>) {
    if cfg!(windows) {
        (
            PathBuf::from(r"C:\Windows\System32\ping.exe"),
            vec!["-n".to_string(), "120".to_string(), "127.0.0.1".to_string()],
        )
    } else {
        (PathBuf::from("/bin/sleep"), vec!["120".to_string()])
    }
}

/// Whether the OS still has this process — the same probe the workbench uses.
fn process_alive(pid: u32) -> bool {
    neo_nexus::supervisor::process_is_live(pid)
}

/// The behaviour the workbench claimed and did not have: `ProcessSupervisor`
/// terminates everything registered when it drops, so a supervisor built inside
/// one request killed the node it had just started, and `stop` on a node started
/// elsewhere only rewrote the row.
#[test]
fn starting_a_node_leaves_a_live_process_that_stop_really_stops() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);

    let (binary, args) = long_running_command();
    let repository = Repository::open(&server.db_path).expect("open workspace");
    let node = repository
        .create_node(NewNode {
            name: "live-node".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: binary,
            args,
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 43332,
            p2p_port: 43333,
            ws_port: None,
        })
        .expect("node creation");

    let started = into_response(
        http.post(&format!("{base}/nodes/{}/start", node.id))
            .set("cookie", &session)
            .set("origin", base)
            .call(),
    );
    assert_eq!(started.status(), 303);
    let location = started.header("location").expect("redirect back to node");
    assert!(
        location.contains("launched%20with%20PID"),
        "the control should report a pid: {location}"
    );

    let running = repository
        .list_nodes()
        .expect("nodes")
        .into_iter()
        .find(|stored| stored.id == node.id)
        .expect("node");
    assert_eq!(running.status, neo_nexus::types::NodeStatus::Running);
    let pid = running.pid.expect("a running node records its pid");
    assert!(
        process_alive(pid),
        "pid {pid} was reported Running but is not alive — the supervisor dropped it"
    );

    let stopped = into_response(
        http.post(&format!("{base}/nodes/{}/stop", node.id))
            .set("cookie", &session)
            .set("origin", base)
            .call(),
    );
    assert_eq!(stopped.status(), 303);
    let location = stopped.header("location").expect("redirect back to node");
    assert!(
        location.contains("stopped") && location.contains("pid"),
        "stop should confirm the process it stopped: {location}"
    );
    assert!(
        !process_alive(pid),
        "pid {pid} outlived the stop that reported success"
    );
    let settled = repository
        .list_nodes()
        .expect("nodes")
        .into_iter()
        .find(|stored| stored.id == node.id)
        .expect("node");
    assert_eq!(settled.status, neo_nexus::types::NodeStatus::Stopped);
    assert_eq!(settled.pid, None, "a stopped node keeps no pid");
}

#[test]
fn deleting_a_running_node_stops_it_before_removing_the_row() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);
    let (binary, args) = long_running_command();
    let repository = Repository::open(&server.db_path).expect("open workspace");
    let node = repository
        .create_node(NewNode {
            name: "delete-live".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: binary,
            args,
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 44332,
            p2p_port: 44333,
            ws_port: None,
        })
        .expect("node creation");
    let started = into_response(
        http.post(&format!("{base}/nodes/{}/start", node.id))
            .set("cookie", &session)
            .set("origin", base)
            .call(),
    );
    assert_eq!(started.status(), 303);
    let running = repository
        .list_nodes()
        .expect("nodes")
        .into_iter()
        .find(|stored| stored.id == node.id)
        .expect("running node");
    let pid = running.pid.expect("running node pid");

    let deleted = post_form_as(
        &http,
        &session,
        &format!("{base}/nodes/{}/delete", node.id),
        "",
    );
    assert_eq!(deleted.status(), 303);
    let location = deleted.header("location").expect("delete redirect");
    assert!(location.contains("deleted"), "delete result: {location}");
    assert!(
        !process_alive(pid),
        "delete removed the only durable handle while pid {pid} stayed alive"
    );
    assert!(repository
        .list_nodes()
        .expect("nodes")
        .iter()
        .all(|stored| stored.id != node.id));
}

#[test]
fn cli_stop_is_a_durable_intent_that_the_web_watchdog_does_not_undo() {
    let server = spawn_supervised_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);
    let repository = Repository::open(&server.db_path).expect("open workspace");
    repository
        .save_watchdog_policy(RestartPolicy::new(
            1,
            Duration::from_secs(1),
            Duration::from_secs(1),
        ))
        .expect("watchdog policy");
    let (binary, args) = long_running_command();
    let node = repository
        .create_node(NewNode {
            name: "cli-stop-witness".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: binary,
            args,
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 45332,
            p2p_port: 45333,
            ws_port: None,
        })
        .expect("node creation");
    assert_eq!(
        into_response(
            http.post(&format!("{base}/nodes/{}/start", node.id))
                .set("cookie", &session)
                .set("origin", base)
                .call(),
        )
        .status(),
        303
    );
    let pid = repository
        .list_nodes()
        .expect("nodes")
        .into_iter()
        .find(|stored| stored.id == node.id)
        .and_then(|stored| stored.pid)
        .expect("running pid");

    let db_arg = server.db_path.display().to_string();
    let stopped =
        neo_nexus::cli::action_from_args(["neo-nexus", "--node-stop", &db_arg, "cli-stop-witness"])
            .expect("CLI stop action");
    assert!(
        matches!(
            stopped,
            neo_nexus::cli::CliAction::PrintWithExitCode { exit_code: 0, .. }
        ),
        "CLI stop failed: {stopped:?}"
    );
    assert!(!process_alive(pid), "CLI stop left pid {pid} alive");
    assert!(
        wait_until(Duration::from_secs(5), || !server
            .state
            .is_supervised(&node.id)),
        "the web supervisor never reaped the CLI-stopped child"
    );
    std::thread::sleep(Duration::from_secs(2));

    let settled = repository
        .list_nodes()
        .expect("nodes")
        .into_iter()
        .find(|stored| stored.id == node.id)
        .expect("node remains registered");
    assert_eq!(settled.status, neo_nexus::types::NodeStatus::Stopped);
    assert_eq!(settled.pid, None);
    let events = repository
        .list_events(RuntimeEventFilter::new(None, "", 100))
        .expect("events");
    assert!(
        events
            .iter()
            .filter(|event| event.node_id.as_deref() == Some(node.id.as_str()))
            .all(|event| event.kind != neo_nexus::events::EventKind::WatchdogScheduled),
        "a deliberate CLI stop was misclassified as a crash: {events:?}"
    );
}

/// The claim the Settings page makes and nothing honoured until now: when a node
/// dies on its own, the workbench notices and brings it back within policy.
#[test]
fn the_watchdog_notices_a_crash_and_restarts_the_node() {
    let server = spawn_supervised_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);

    // One attempt, one second from now: fast enough to observe, bounded enough
    // that a crash loop cannot run forever during a test.
    let repository = Repository::open(&server.db_path).expect("open workspace");
    repository
        .save_watchdog_policy(RestartPolicy::new(
            1,
            Duration::from_secs(1),
            Duration::from_secs(1),
        ))
        .expect("watchdog policy");

    let (binary, args) = crashing_command();
    let node = repository
        .create_node(NewNode {
            name: "crasher".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Testnet,
            binary_path: binary,
            args,
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 46332,
            p2p_port: 46333,
            ws_port: None,
        })
        .expect("node creation");

    let started = into_response(
        http.post(&format!("{base}/nodes/{}/start", node.id))
            .set("cookie", &session)
            .set("origin", base)
            .call(),
    );
    assert_eq!(started.status(), 303);

    let kinds = || -> Vec<String> {
        repository
            .list_events(RuntimeEventFilter::new(None, "", 100))
            .unwrap_or_default()
            .iter()
            .map(|event| format!("{}:: {}", event.kind, event.message))
            .collect()
    };

    assert!(
        wait_until(Duration::from_secs(20), || {
            let seen = kinds();
            seen.iter()
                .any(|entry| entry.starts_with("watchdog-scheduled"))
                && seen
                    .iter()
                    .any(|entry| entry.starts_with("watchdog-restarted"))
        }),
        "the watchdog never noticed the crash; journal was {:?}",
        kinds()
    );

    // The node must not be left claiming to run.
    let final_status = repository
        .list_nodes()
        .expect("nodes")
        .into_iter()
        .find(|stored| stored.id == node.id)
        .expect("node");
    assert!(
        matches!(
            final_status.status,
            neo_nexus::types::NodeStatus::Error | neo_nexus::types::NodeStatus::Running
        ),
        "unexpected status {:?}",
        final_status.status
    );
}

/// A node recorded Running that this server holds no handle for must not stay
/// Running once its process is gone.
#[test]
fn an_unmanaged_node_is_settled_once_its_process_disappears() {
    let server = spawn_supervised_server();
    let repository = Repository::open(&server.db_path).expect("open workspace");
    let (binary, args) = crashing_command();
    let node = repository
        .create_node(NewNode {
            name: "ghost".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Testnet,
            binary_path: binary,
            args,
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 47332,
            p2p_port: 47333,
            ws_port: None,
        })
        .expect("node creation");

    // A pid that cannot exist: the row claims Running, nothing backs it.
    repository
        .update_node_status(
            &node.id,
            neo_nexus::types::NodeStatus::Running,
            Some(4_000_000),
        )
        .expect("seed status");

    let settled = wait_until(Duration::from_secs(10), || {
        repository
            .list_nodes()
            .unwrap_or_default()
            .iter()
            .find(|stored| stored.id == node.id)
            .is_some_and(|stored| stored.status == neo_nexus::types::NodeStatus::Stopped)
    });
    assert!(settled, "a stale Running row was never settled");
}

/// The install endpoint is a host-mutating action, so it cannot be reachable
/// without a session, and a bad request must fail visibly rather than quietly.
#[test]
fn runtime_install_requires_a_session_and_reports_a_refused_job() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;

    let anonymous = into_response(
        http.post(&format!("{base}/runtimes/install"))
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string("profile=ghost&release=rel-1"),
    );
    assert_eq!(
        anonymous.status(),
        303,
        "an unauthenticated install must redirect to sign in"
    );
    assert_eq!(
        anonymous.header("location"),
        Some("/login"),
        "and not be accepted"
    );

    let session = signed_in(&http, base);
    let accepted = post_form_as(
        &http,
        &session,
        &format!("{base}/runtimes/install"),
        "profile=ghost&release=rel-1",
    );
    assert_eq!(accepted.status(), 303);
    let location = accepted.header("location").expect("redirect back");
    assert!(
        location.contains("install%20started"),
        "the page should acknowledge the job it queued: {location}"
    );

    // The job runs off the request thread, so the outcome has to be observed.
    let mut settled = String::new();
    for _ in 0..100 {
        let page = into_response(
            http.get(&format!("{base}/runtimes"))
                .set("cookie", &session)
                .call(),
        )
        .into_string()
        .expect("runtimes page");
        if page.contains("failed") {
            settled = page;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(
        settled.contains("failed"),
        "the job never reported a result to the page"
    );
    assert!(
        settled.contains("ghost"),
        "the failure should name the profile that could not be found: {settled}"
    );

    let repository = Repository::open(&server.db_path).expect("reopen workspace");
    assert!(
        repository
            .list_runtime_installations()
            .expect("installations")
            .is_empty(),
        "a refused install must not record anything"
    );
}

/// Catalogue browsing must not be reachable anonymously either: it reaches out
/// to a configured source on the server's behalf.
#[test]
fn runtime_catalogue_browsing_requires_a_session() {
    let server = spawn_server();
    let http = agent();
    let anonymous = into_response(
        http.get(&format!("{}/runtimes?profile=cat-1", server.base_url))
            .call(),
    );
    assert_eq!(anonymous.status(), 303);

    let session = signed_in(&http, &server.base_url);
    let page = into_response(
        http.get(&format!("{}/runtimes?profile=cat-1", server.base_url))
            .set("cookie", &session)
            .call(),
    );
    assert_eq!(page.status(), 200);
    assert!(
        page.into_string()
            .expect("page body")
            .contains("no longer exists"),
        "an unknown profile should be stated plainly"
    );
}

/// The lifecycle controls must leave an audit trail. The web workbench restored
/// start/stop/restart but not the journal entries the desktop shell wrote, so
/// "who stopped this node at 03:00" had no answer.
#[test]
fn lifecycle_controls_are_journaled() {
    let server = spawn_supervised_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);

    let (binary, args) = long_running_command();
    let repository = Repository::open(&server.db_path).expect("open workspace");
    let node = repository
        .create_node(NewNode {
            name: "audited".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: binary,
            args,
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 52332,
            p2p_port: 52333,
            ws_port: None,
        })
        .expect("node creation");

    for verb in ["start", "stop"] {
        let response = into_response(
            http.post(&format!("{base}/nodes/{}/{verb}", node.id))
                .set("cookie", &session)
                .set("origin", base)
                .call(),
        );
        assert_eq!(response.status(), 303, "{verb} should be accepted");
    }

    let journal = repository
        .list_events(RuntimeEventFilter::new(None, "", 100))
        .expect("events")
        .iter()
        .map(|event| format!("{}:: {}", event.kind, event.message))
        .collect::<Vec<_>>();
    let text = journal.join("\n");

    assert!(
        text.contains("node-started:: audited launched with PID"),
        "no start entry; journal was:\n{text}"
    );
    assert!(
        text.contains("node-stopped:: audited stopped"),
        "no stop entry; journal was:\n{text}"
    );
    // Both entries carry the node id, so the trail is attributable rather than
    // just present. Registration is covered by the create/edit/delete test; this
    // node was inserted straight through the repository and so has no such entry.
    let attributable = journal
        .iter()
        .filter(|entry| entry.contains("audited"))
        .count();
    assert_eq!(
        attributable, 2,
        "both lifecycle entries should name the node; journal was:\n{text}"
    );
}

/// Policy changes need a trail too: a schedule that silently changed is an
/// incident waiting to be reconstructed.
#[test]
fn policy_saves_are_journaled() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);

    post_form_as(
        &http,
        &session,
        &format!("{base}/settings/watchdog"),
        "enabled=Enabled&max_restart_attempts=5&base_delay_seconds=4&max_delay_seconds=60",
    );
    let repository = Repository::open(&server.db_path).expect("reopen workspace");
    let journal = repository
        .list_events(RuntimeEventFilter::new(None, "", 50))
        .expect("events")
        .iter()
        .map(|event| event.kind.to_string())
        .collect::<Vec<_>>();
    assert!(
        journal.contains(&"watchdog-policy-updated".to_string()),
        "a policy change wrote no journal entry; got {journal:?}"
    );
}

// -- custody: the signer surface --------------------------------------------

/// What this suite can assert now, and what it deliberately no longer tries to.
///
/// §7 step 2 made the workbench a *client* of the custody service, and the
/// questions changed with it. "Does the policy engine refuse the right bytes on
/// the right network" belongs to `workers/neo-signer`, which answers it holding
/// keys it actually sealed; a copy of that here would only be testing a stub.
/// What is answerable nowhere else is the trip: whose credential reached the
/// service, what this process added to or subtracted from a caller's request, and
/// whether an operator reading a page is reading what the service stored or what
/// they typed.
///
/// So every answer below is scripted by the test asserting on it — including
/// refusal codes and status codes this crate has never heard of, which is the
/// point: the relay has to survive a service newer than itself. A route nobody
/// scripted answers 501 with a code of its own, so an unexpected request reads
/// as the omission it is instead of passing.
struct StubService {
    base_url: String,
    rules: Arc<Mutex<Vec<Rule>>>,
    requests: Arc<Mutex<Vec<String>>>,
}

/// One scripted route. `answers` queues, so a test can make the same route say
/// three different things in order; the last answer repeats once the queue is
/// spent, because a page that renders twice is not a test failure.
struct Rule {
    /// `METHOD /path`, compared with any query string removed.
    key: String,
    answers: VecDeque<(u16, String, Duration)>,
}

/// The credential the console is configured with. Distinct from every caller
/// token a test presents, which is what makes "whose credential reached the
/// service?" an assertion rather than a hope.
const ADMIN_CREDENTIAL: &str = "nsk1_console_admin_1f4e";
/// A caller's own credential, presented to the console by a test acting as a
/// program. Lowercase like every credential in this file, because the stub
/// lowercases each header line it records.
const CALLER_CREDENTIAL: &str = "nsk1_caller_relayer_7c02";

const KEY_ID: &str = "key-8f14e45fceea167a5a36dedd4bea2543";
const CALLER_ID: &str = "caller-9f31e45fceea167a5a36dedd4bea2543";

const KEY_ROW: &str = r#"{"key_id":"key-8f14e45fceea167a5a36dedd4bea2543","label":"treasury","network":"testnet","public_key":"02ab","script_hash":"0xef4073a0f2bacd0bc1d5de799e3b661d633eb9f9","address":"NdhLie6L7CiGvSZ39jyCz6mVmWE3dN6RfD","verification_script":"0c2102ab","signing_enabled":true}"#;
const CALLER_ROW: &str = r#"{"id":"caller-9f31e45fceea167a5a36dedd4bea2543","label":"relayer","auth_mode":"bearer","workload_public_key":null,"workload_subject":null,"key_grant":{"mode":"any","key_ids":[]},"capabilities":["sign"],"allowed_origins":[],"created_at_unix":1755000000,"disabled":false}"#;
const WORKLOAD_CALLER_ID: &str = "caller-7c02e45fceea167a5a36dedd4bea2543";
const WORKLOAD_CALLER_ROW: &str = r#"{"id":"caller-7c02e45fceea167a5a36dedd4bea2543","label":"production relayer","auth_mode":"workload-ed25519","workload_public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","workload_subject":"relayer-prod","key_grant":{"mode":"only","key_ids":["key-8f14e45fceea167a5a36dedd4bea2543"]},"capabilities":["sign"],"allowed_origins":[],"created_at_unix":1755000001,"disabled":false,"future_attestation":{"pcr0":"bb"}}"#;
const FUTURE_CALLER_ID: &str = "caller-6b01e45fceea167a5a36dedd4bea2543";
const FUTURE_CALLER_ROW: &str = r#"{"id":"caller-6b01e45fceea167a5a36dedd4bea2543","label":"future identity","auth_mode":"mtls-spiffe","workload_public_key":null,"workload_subject":"spiffe://neo/relayer","key_grant":{"mode":"only","key_ids":[]},"capabilities":["sign"],"allowed_origins":[],"created_at_unix":1755000002,"disabled":false}"#;

impl StubService {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a free loopback port");
        let address = listener.local_addr().expect("bound address");
        let stub = StubService {
            base_url: format!("http://{address}"),
            rules: Arc::new(Mutex::new(Vec::new())),
            requests: Arc::new(Mutex::new(Vec::new())),
        };
        let (rules, requests) = (stub.rules.clone(), stub.requests.clone());
        // One thread per stub, on its own port, serving until the process ends.
        // A test that stops making requests leaves it parked in `accept`, which
        // costs nothing and is what a service with no client looks like anyway.
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut stream = stream;
                let Ok(request) = read_stub_request(&mut stream) else {
                    continue;
                };
                let (status, body, delay) = answer(&rules, &request);
                requests.lock().expect("unlocked").push(request);
                thread::sleep(delay);
                let head = format!(
                    "HTTP/1.1 {status} Answer\r\nContent-Type: application/json\r\nContent-Length: \
                     {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                if stream.write_all(head.as_bytes()).is_err()
                    || stream.write_all(body.as_bytes()).is_err()
                {
                    continue;
                }
            }
        });
        stub
    }

    /// Script the next answer to `METHOD /path`.
    fn on(&self, key: &str, status: u16, body: &str) -> &Self {
        self.on_delayed(key, status, body, Duration::ZERO)
    }

    fn on_delayed(&self, key: &str, status: u16, body: &str, delay: Duration) -> &Self {
        let mut rules = self.rules.lock().expect("unlocked");
        match rules.iter_mut().find(|rule| rule.key == key) {
            Some(rule) => rule.answers.push_back((status, body.to_string(), delay)),
            None => rules.push(Rule {
                key: key.to_string(),
                answers: VecDeque::from([(status, body.to_string(), delay)]),
            }),
        }
        self
    }

    /// The three reads the key list makes, in whatever order the page asks.
    fn with_inventory(&self) -> &Self {
        self.on(
            "GET /signer/api/v1/keys",
            200,
            &format!(r#"{{"allowed":true,"keys":[{KEY_ROW}]}}"#),
        )
        .on(
            "GET /signer/api/v1/callers",
            200,
            &format!(r#"{{"allowed":true,"callers":[{CALLER_ROW}]}}"#),
        )
        .on(
            "GET /signer/api/v1/audit",
            200,
            r#"{"allowed":true,"entries":[]}"#,
        )
    }

    /// Every request the console made, in order.
    fn requests(&self) -> Vec<String> {
        self.requests.lock().expect("unlocked").clone()
    }

    /// How many of them named `METHOD /path`.
    fn asked(&self, key: &str) -> usize {
        self.requests()
            .iter()
            .filter(|request| first_line(request) == key)
            .count()
    }
}

/// The `METHOD /path` of a recorded request, with the HTTP version and any query
/// string removed, so a route can be named without spelling out the filter the
/// console happened to send. A line this cannot read yields a key that matches no
/// rule, and the stub's default refusal says so.
fn first_line(request: &str) -> String {
    let line = request.lines().next().unwrap_or("");
    let mut parts = line.split(' ');
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");
    format!("{method} {}", path.split('?').next().unwrap_or(""))
}

/// The scripted answer for a request, or a refusal loud enough to read as the
/// test's own omission rather than as the console's behaviour.
fn answer(rules: &Arc<Mutex<Vec<Rule>>>, request: &str) -> (u16, String, Duration) {
    let key = first_line(request);
    let mut rules = rules.lock().expect("unlocked");
    let Some(rule) = rules.iter_mut().find(|rule| rule.key == key) else {
        return (
            501,
            r#"{"allowed":false,"code":"stub-has-no-answer","message":"the test scripted no reply for this route"}"#.to_string(),
            Duration::ZERO,
        );
    };
    if rule.answers.len() > 1 {
        rule.answers.pop_front().expect("length checked")
    } else {
        rule.answers.front().cloned().unwrap_or((
            501,
            r#"{"allowed":false,"code":"stub-emptied"}"#.to_string(),
            Duration::ZERO,
        ))
    }
}

/// The request as one string: the request line untouched, each header line
/// lowercased so an assertion is about presence rather than capitalization, and
/// the body verbatim — which is the half a relay test actually needs.
fn read_stub_request(stream: &mut std::net::TcpStream) -> std::io::Result<String> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    let mut content_length = 0_usize;
    let mut is_head = true;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if let Some(value) = line
            .split_once(':')
            .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
            .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        {
            content_length = value;
        }
        if is_head {
            is_head = false;
            request.push_str(&line);
        } else {
            request.push_str(&line.to_lowercase());
        }
        if line == "\r\n" {
            break;
        }
    }
    if content_length > 0 {
        let mut body = vec![0_u8; content_length];
        reader.read_exact(&mut body)?;
        request.push_str(&String::from_utf8_lossy(&body));
    }
    Ok(request)
}

/// A console wired to the stub the way a deployment wires it to the service.
fn spawn_custody_server(stub: &StubService) -> Server {
    spawn_custody_server_with_relay_limit(stub, None)
}

fn spawn_custody_server_with_relay_limit(stub: &StubService, relay_limit: Option<usize>) -> Server {
    let config = SignerConfig::new_insecure_loopback_for_test(
        &stub.base_url,
        Some(ADMIN_CREDENTIAL.to_string()),
        Duration::from_secs(5),
    )
    .expect("a loopback URL is a valid custody configuration");
    spawn_server_custody_with_relay_limit(
        Custody::serving(SignerClient::new(config)).expect("valid stub custody"),
        relay_limit,
    )
}

/// POST a signing request the way a bearer-authenticated program does: optional
/// browser `Origin`, and no operator session cookie — a caller has none.
fn post_sign(
    http: &ureq::Agent,
    url: &str,
    token: Option<&str>,
    origin: Option<&str>,
    key_id: &str,
    unsigned_hex: &str,
) -> ureq::Response {
    let mut request = http.post(url);
    if let Some(token) = token {
        request = request.set("authorization", &format!("Bearer {token}"));
    }
    if let Some(origin) = origin {
        request = request.set("origin", origin);
    }
    let body = serde_json::json!({ "key_id": key_id, "unsigned_hex": unsigned_hex }).to_string();
    into_response(
        request
            .set("content-type", "application/json")
            .send_string(&body),
    )
}

fn sign_url(server: &Server) -> String {
    format!("{}/signer/api/v1/sign/transaction", server.base_url)
}

fn consensus_url(server: &Server) -> String {
    format!("{}/signer/api/v1/sign/consensus", server.base_url)
}

fn raw_url(server: &Server) -> String {
    format!("{}/signer/api/v1/sign/raw", server.base_url)
}

fn eip191_url(server: &Server) -> String {
    format!("{}/signer/api/v1/sign/eip191-fulfillment", server.base_url)
}

fn post_raw(
    http: &ureq::Agent,
    url: &str,
    token: &str,
    key_id: &str,
    data_hex: &str,
) -> ureq::Response {
    into_response(
        http.post(url)
            .set("authorization", &format!("Bearer {token}"))
            .set("content-type", "application/json")
            .send_string(
                &serde_json::json!({ "key_id": key_id, "data_hex": data_hex }).to_string(),
            ),
    )
}

#[test]
fn custody_pages_need_a_session_and_the_signing_api_does_not() {
    let stub = StubService::start();
    stub.with_inventory();
    // §5's own answer to a request that carried no credential. The console
    // forwards the missing credential rather than deciding for itself that
    // nobody is here: a refusal made before the service saw the request never
    // reaches the audit trail the operator reads.
    stub.on(
        "POST /signer/api/v1/sign/transaction",
        403,
        r#"{"allowed":false,"code":"missing-token","message":"no bearer token was presented"}"#,
    );
    let server = spawn_custody_server(&stub);
    let http = agent();
    let base = &server.base_url;

    let anonymous = into_response(http.get(&format!("{base}/signer")).call());
    assert_eq!(anonymous.status(), 303);
    assert_eq!(anonymous.header("location"), Some("/login"));

    let response = post_sign(&http, &sign_url(&server), None, None, KEY_ID, "7a7a");
    assert_eq!(response.status(), 403);
    assert_eq!(
        stub.asked("POST /signer/api/v1/sign/transaction"),
        1,
        "the session layer answered a caller that never has a session"
    );
    let body = json_body(response);
    assert_eq!(body["code"], "missing-token");
    assert_eq!(body["allowed"], false);
    // The caller reads a code and a sentence of its own. The specifics — which
    // blacklist, which amount — stay in the vault.
    assert!(body.get("detail").is_none(), "{body}");
    let requests = stub.requests();
    let forwarded = &requests[0];
    assert!(
        !forwarded.contains(ADMIN_CREDENTIAL),
        "the relay signed the caller's request with the console's own credential: {forwarded}"
    );

    let page = into_response(
        http.get(&format!("{base}/signer"))
            .set("cookie", &signed_in(&http, base))
            .call(),
    );
    assert_eq!(page.status(), 200);
    let text = page.into_string().expect("utf-8 body");
    assert!(text.contains("Remote custody connected"), "{text}");
    assert!(text.contains("holds only its admin credential"), "{text}");
    // The inventory is the service's rows, and the page says so rather than
    // looking like an empty vault.
    assert!(text.contains("treasury"), "{text}");
    assert!(text.contains("relayer"), "{text}");
    // The configured URL is plain HTTP, which the page warns about instead of
    // refusing: §6 assumes loopback, and this is loopback.
    assert!(text.contains("plain HTTP"), "{text}");
}

#[test]
fn public_signer_relay_rejects_work_beyond_its_strict_concurrency_bound() {
    let stub = StubService::start();
    stub.on_delayed(
        "POST /signer/api/v1/sign/transaction",
        403,
        r#"{"allowed":false,"code":"missing-token","message":"no bearer token was presented"}"#,
        Duration::from_millis(600),
    );
    let server = spawn_custody_server_with_relay_limit(&stub, Some(1));
    let first_url = sign_url(&server);
    let first =
        thread::spawn(move || post_sign(&agent(), &first_url, None, None, KEY_ID, "7a7a").status());
    assert!(wait_until(Duration::from_secs(2), || {
        stub.asked("POST /signer/api/v1/sign/transaction") == 1
    }));

    let started = Instant::now();
    let rejected = post_sign(&agent(), &sign_url(&server), None, None, KEY_ID, "7a7a");
    assert_eq!(rejected.status(), 429);
    assert_eq!(rejected.header("retry-after"), Some("1"));
    assert!(
        started.elapsed() < Duration::from_millis(300),
        "the excess relay waited behind blocking work"
    );
    let body = json_body(rejected);
    assert_eq!(body["code"], "signer-relay-busy");
    assert_eq!(
        stub.asked("POST /signer/api/v1/sign/transaction"),
        1,
        "rejected work still reached custody"
    );

    assert_eq!(first.join().expect("first relay finished"), 403);
    let admitted_again = post_sign(&agent(), &sign_url(&server), None, None, KEY_ID, "7a7a");
    assert_eq!(admitted_again.status(), 403);
    assert_eq!(stub.asked("POST /signer/api/v1/sign/transaction"), 2);
}

#[test]
fn a_refusal_is_the_services_answer_including_a_status_nobody_has_seen() {
    let stub = StubService::start();
    stub.on("POST /signer/api/v1/sign/transaction", 403, r#"{"allowed":false,"code":"unknown-token","message":"this credential is not registered"}"#)
        .on("POST /signer/api/v1/sign/transaction", 400, r#"{"allowed":false,"code":"signer-transaction-unparsable","message":"the bytes are not a transaction this parser knows"}"#)
        // A code and a status from a service release newer than this client. §5's
        // code→status table belongs to the service, and the relay that kept its
        // own copy would turn every new code into a wrong status here.
        .on("POST /signer/api/v1/sign/transaction", 429, r#"{"allowed":false,"code":"caller-rate-limited","message":"too many requests for this caller"}"#)
        .on("POST /signer/api/v1/sign/transaction", 503, r#"{"allowed":false,"code":"signer-service-busy","message":"the signer is temporarily busy; retry later"}"#);
    let server = spawn_custody_server(&stub);
    let http = agent();

    let scripted: [(u16, &str, &str, Option<&str>); 4] = [
        (
            403,
            "unknown-token",
            "this credential is not registered",
            None,
        ),
        (
            400,
            "signer-transaction-unparsable",
            "the bytes are not a transaction this parser knows",
            None,
        ),
        (
            429,
            "caller-rate-limited",
            "too many requests for this caller",
            None,
        ),
        (
            503,
            "signer-service-busy",
            "the signer is temporarily busy; retry later",
            Some("1"),
        ),
    ];
    for (index, (status, code, message, retry_after)) in scripted.iter().enumerate() {
        let response = post_sign(
            &http,
            &sign_url(&server),
            Some(CALLER_CREDENTIAL),
            None,
            KEY_ID,
            "7a7a",
        );
        assert_eq!(response.status(), *status, "answer #{index}");
        assert_eq!(
            response.header("retry-after"),
            *retry_after,
            "answer #{index}"
        );
        let body = json_body(response);
        assert_eq!(body["code"], *code, "{body}");
        assert_eq!(body["message"], *message, "{body}");
        assert_eq!(body["allowed"], false, "{body}");
    }

    // The credential that arrived is the credential that left. The console's
    // admin token would be refused on a sign route anyway, and a relay that
    // substituted it would attribute every signature in the trail to whoever ran
    // the browser.
    for request in stub.requests() {
        assert!(
            request.contains(&format!("authorization: bearer {CALLER_CREDENTIAL}")),
            "{request}"
        );
        assert!(!request.contains(ADMIN_CREDENTIAL), "{request}");
    }
}

#[test]
fn an_allowed_answer_carries_the_services_witness_unchanged() {
    let stub = StubService::start();
    stub.on(
        "POST /signer/api/v1/sign/transaction",
        200,
        r#"{"allowed":true,"key_id":"key-8f14e45fceea167a5a36dedd4bea2543","script_hash":"0xef4073a0f2bacd0bc1d5de799e3b661d633eb9f9","address":"NdhLie6L7CiGvSZ39jyCz6mVmWE3dN6RfD","digest":"aabb","invocation_script":"0c40a1b2","verification_script":"0c2102ab"}"#,
    );
    let server = spawn_custody_server(&stub);

    let body = json_body(post_sign(
        &agent(),
        &sign_url(&server),
        Some(CALLER_CREDENTIAL),
        None,
        KEY_ID,
        "7a7a",
    ));
    assert_eq!(body["allowed"], true);
    // The witness is the service's, byte for byte, and `"allowed": true` sits
    // beside it rather than wrapping it: the payload fields arrive flattened, so
    // a caller parses this exactly as it parses the service directly.
    assert_eq!(body["invocation_script"], "0c40a1b2");
    assert_eq!(body["verification_script"], "0c2102ab");
    assert_eq!(body["key_id"], KEY_ID);
    assert_eq!(body["address"], "NdhLie6L7CiGvSZ39jyCz6mVmWE3dN6RfD");
}

#[test]
fn workload_relay_preserves_exact_body_route_and_authentication_headers() {
    let stub = StubService::start();
    stub.on(
        "POST /signer/api/v1/sign/transaction",
        403,
        r#"{"allowed":false,"code":"unknown-workload","message":"test assertion is intentionally synthetic"}"#,
    );
    let server = spawn_custody_server(&stub);
    let body = format!("{{\n  \"unsigned_hex\" : \"7a7a\",  \"key_id\" : \"{KEY_ID}\"\n}}");
    let signature = "ab".repeat(64);
    let url = format!("{}?trace=alpha%2Fbeta&attempt=1", sign_url(&server));
    let response = into_response(
        agent()
            .post(&url)
            .set("content-type", "application/json")
            .set("x-neoos-workload-protocol", "neoos-workload-v2")
            .set("x-neoos-audience", "https://custody.example")
            .set("x-neoos-caller", "relayer-workload-1")
            .set("x-neoos-timestamp", "1755000123")
            .set("x-neoos-nonce", "nonce_0123456789abcdef")
            .set("x-neoos-signature", &signature)
            .set("origin", "https://relayer.example")
            .set("referer", "https://relayer.example/jobs/42")
            .set("cookie", "neo_nexus_session=must-not-cross")
            .set("x-forwarded-for", "203.0.113.9")
            .set("x-not-a-signer-header", "must-not-cross")
            .send_string(&body),
    );
    assert_eq!(response.status(), 403);
    assert_eq!(json_body(response)["code"], "unknown-workload");

    let forwarded = stub.requests().pop().expect("the workload relay asked");
    assert!(
        forwarded
            .starts_with("POST /signer/api/v1/sign/transaction?trace=alpha%2Fbeta&attempt=1 HTTP"),
        "{forwarded}"
    );
    assert_eq!(
        forwarded
            .split_once("\r\n\r\n")
            .expect("forwarded request has a body")
            .1,
        body,
        "workload authentication hashes the exact HTTP body bytes"
    );
    for expected in [
        "x-neoos-workload-protocol: neoos-workload-v2",
        "x-neoos-audience: https://custody.example",
        "x-neoos-caller: relayer-workload-1",
        "x-neoos-timestamp: 1755000123",
        "x-neoos-nonce: nonce_0123456789abcdef",
        &format!("x-neoos-signature: {signature}"),
        "origin: https://relayer.example",
        "referer: https://relayer.example/jobs/42",
    ] {
        assert!(
            forwarded.contains(expected),
            "missing {expected}: {forwarded}"
        );
    }
    for forbidden in [
        ADMIN_CREDENTIAL,
        "authorization:",
        "neo_nexus_session",
        "x-forwarded-for:",
        "x-not-a-signer-header:",
    ] {
        assert!(
            !forwarded.contains(forbidden),
            "{forbidden} crossed the signer relay allowlist: {forwarded}"
        );
    }
}

#[test]
fn signer_relay_rejects_an_oversized_body_before_custody() {
    let stub = StubService::start();
    let server = spawn_custody_server(&stub);
    let body = vec![b'x'; 256 * 1024 + 1];
    let response = into_response(
        agent()
            .post(&sign_url(&server))
            .set("authorization", &format!("Bearer {CALLER_CREDENTIAL}"))
            .set("content-type", "application/json")
            .send_bytes(&body),
    );
    assert_eq!(response.status(), 413);
    assert_eq!(json_body(response)["code"], "signer-request-too-large");
    assert!(
        stub.requests().is_empty(),
        "an oversized body reached custody: {:?}",
        stub.requests()
    );
}

#[test]
fn a_neox_request_and_response_cross_the_proxy_without_losing_fields() {
    let stub = StubService::start();
    stub.on(
        "POST /signer/api/v1/sign/transaction",
        200,
        r#"{"allowed":true,"key_id":"key-8f14e45fceea167a5a36dedd4bea2543","script_hash":"0x1234","address":"0xabcd","digest":"aabb","chain_family":"neox","chain_id":47763,"signed_transaction":"02f8","signature":"11","public_key":"04ab","future_receipt":{"type":2}}"#,
    );
    let server = spawn_custody_server(&stub);
    let request = serde_json::json!({
        "key_id": KEY_ID,
        "unsigned_hex": "02f8",
        "request_id": "request-42",
        "chain_family": "neox",
        "chain_id": 47_763,
    });
    let response = into_response(
        agent()
            .post(&sign_url(&server))
            .set("authorization", &format!("Bearer {CALLER_CREDENTIAL}"))
            .set("content-type", "application/json")
            .send_string(&request.to_string()),
    );
    assert_eq!(response.status(), 200);
    let body = json_body(response);
    assert_eq!(body["allowed"], true);
    assert_eq!(body["chain_family"], "neox");
    assert_eq!(body["chain_id"], 47_763);
    assert_eq!(body["signed_transaction"], "02f8");
    assert_eq!(body["signature"], "11");
    assert_eq!(body["public_key"], "04ab");
    assert_eq!(body["future_receipt"]["type"], 2);
    assert!(body.get("invocation_script").is_none());
    assert!(body.get("verification_script").is_none());

    let forwarded = stub.requests().pop().expect("the signer was called");
    assert!(
        forwarded.contains(r#""request_id":"request-42""#),
        "{forwarded}"
    );
    assert!(
        forwarded.contains(r#""chain_family":"neox""#),
        "{forwarded}"
    );
    assert!(forwarded.contains(r#""chain_id":47763"#), "{forwarded}");
}

#[test]
fn signer_documentation_covers_every_supported_v1_route() {
    let document = include_str!("../docs/signer-service-design.md");
    let reference = document
        .split_once("### API Reference")
        .expect("signer design has an API Reference")
        .1
        .split_once("### Usage Examples")
        .expect("API Reference precedes Usage Examples")
        .0;
    let routes = [
        "POST /sign/transaction",
        "POST /sign/consensus",
        "POST /sign/eip191-fulfillment",
        "POST /sign/raw",
        "GET /keys/{id}",
        "POST /keys",
        "GET /keys",
        "POST /keys/{id}/state",
        "DELETE /keys/{id}",
        "GET /keys/{id}/policy",
        "POST /keys/{id}/policy",
        "POST /callers",
        "POST /callers/workload",
        "GET /callers",
        "POST /callers/{id}/rotate",
        "POST /callers/{id}/state",
        "DELETE /callers/{id}",
        "GET /audit",
    ];

    for route in routes {
        assert!(
            reference.contains(&format!("`{route}`")),
            "current API Reference does not name supported v1 route {route}"
        );
    }
    assert!(
        document.contains("18/18（100%）"),
        "documentation coverage metric must remain explicit"
    );
    for unsupported in ["POST /keys/import", "POST /keys/import-nep2"] {
        assert!(
            reference.contains(&format!("`{unsupported}`")),
            "current API Reference must explicitly refuse secret ingress route {unsupported}"
        );
    }
}

#[test]
fn every_current_v1_caller_route_is_wired_to_the_service() {
    let stub = StubService::start();
    stub.on(
        "POST /signer/api/v1/sign/consensus",
        403,
        r#"{"allowed":false,"code":"consensus-forbidden","message":"closed"}"#,
    )
    .on(
        "POST /signer/api/v1/sign/eip191-fulfillment",
        200,
        r#"{"allowed":true,"key_id":"key-8f14e45fceea167a5a36dedd4bea2543","address":"0x1234","public_key":"02ab","digest":"0x01","message_hash":"0x02","signature":"0x03","chain_family":"neox","chain_id":47763,"oracle_contract":"0x2222222222222222222222222222222222222222","future_proof":{"version":2}}"#,
    )
    .on(
        "POST /signer/api/v1/sign/raw",
        200,
        r#"{"allowed":true,"key_id":"key-8f14e45fceea167a5a36dedd4bea2543","script_hash":"0xef4073a0f2bacd0bc1d5de799e3b661d633eb9f9","address":"NdhLie6L7CiGvSZ39jyCz6mVmWE3dN6RfD","digest":"aabb","signature":"11","public_key":"02ab","invocation_script":"0c4011","verification_script":"0c2102ab"}"#,
    )
    .on(
        &format!("GET /signer/api/v1/keys/{KEY_ID}"),
        200,
        &format!(
            r#"{{"allowed":true,{},"chain_family":"neox","chain_id":47763,"future_attestation":{{"pcr0":"aa"}}}}"#,
            &KEY_ROW[1..KEY_ROW.len() - 1]
        ),
    );
    let server = spawn_custody_server(&stub);
    let http = agent();

    let consensus = post_sign(
        &http,
        &consensus_url(&server),
        Some(CALLER_CREDENTIAL),
        None,
        KEY_ID,
        "00",
    );
    assert_eq!(consensus.status(), 403);
    assert_eq!(json_body(consensus)["code"], "consensus-forbidden");

    let raw = json_body(post_raw(
        &http,
        &raw_url(&server),
        CALLER_CREDENTIAL,
        KEY_ID,
        "cafe",
    ));
    assert_eq!(raw["allowed"], true);
    assert_eq!(raw["signature"], "11");
    assert_eq!(raw["public_key"], "02ab");

    let semantic = json_body(into_response(
        http.post(&eip191_url(&server))
            .set("authorization", &format!("Bearer {CALLER_CREDENTIAL}"))
            .set("content-type", "application/json")
            .send_string(
                &serde_json::json!({
                    "key_id": KEY_ID,
                    "request_id": "relayer:neox:7",
                    "chain_id": 47_763,
                    "oracle_contract": "0x2222222222222222222222222222222222222222",
                    "fulfillment": {
                        "request_id": "7",
                        "app_id": "app:1",
                        "module_id": "oracle.fetch",
                        "operation": "privacy_oracle",
                        "success": true,
                        "error": ""
                    },
                    "result_bytes_hex": "0x68656c6c6f"
                })
                .to_string(),
            ),
    ));
    assert_eq!(semantic["allowed"], true);
    assert_eq!(semantic["chain_family"], "neox");
    assert_eq!(semantic["message_hash"], "0x02");
    assert_eq!(semantic["future_proof"]["version"], 2);

    let key = json_body(into_response(
        http.get(&format!("{}/signer/api/v1/keys/{KEY_ID}", server.base_url))
            .set("authorization", &format!("Bearer {CALLER_CREDENTIAL}"))
            .call(),
    ));
    assert_eq!(key["allowed"], true);
    assert_eq!(key["key_id"], KEY_ID);
    assert_eq!(key["chain_family"], "neox");
    assert_eq!(key["chain_id"], 47_763);
    assert_eq!(key["future_attestation"]["pcr0"], "aa");

    let requests = stub.requests();
    assert!(
        requests
            .iter()
            .any(|request| request.contains(r#""data_hex":"cafe""#)),
        "{requests:?}"
    );
    assert!(
        requests.iter().any(|request| {
            request.starts_with("POST /signer/api/v1/sign/eip191-fulfillment")
                && request.contains(r#""result_bytes_hex":"0x68656c6c6f""#)
                && !request.contains(r#""digest""#)
                && !request.contains(r#""message_hash""#)
        }),
        "{requests:?}"
    );
    assert!(
        requests
            .iter()
            .all(|request| !request.contains(ADMIN_CREDENTIAL)),
        "a caller route used the control-plane token: {requests:?}"
    );
}

#[test]
fn signer_service_owns_schema_refusals_and_receives_the_exact_body() {
    let stub = StubService::start();
    for _ in 0..3 {
        stub.on(
            "POST /signer/api/v1/sign/transaction",
            400,
            r#"{"allowed":false,"code":"signer-request-unreadable","message":"the signer could not parse the request"}"#,
        );
    }
    let server = spawn_custody_server(&stub);
    let http = agent();

    let bodies = [
        "{not-json",
        r#"{"key_id":"key-1","unsigned_hex":"00","unexpected":true}"#,
        r#"{"key_id":"key-1"}"#,
    ];
    for body in bodies {
        let response = into_response(
            http.post(&sign_url(&server))
                .set("authorization", &format!("Bearer {CALLER_CREDENTIAL}"))
                .set("content-type", "application/json")
                .send_string(body),
        );
        assert_eq!(response.status(), 400);
        let answer = json_body(response);
        assert_eq!(answer["allowed"], false);
        assert_eq!(answer["code"], "signer-request-unreadable");
    }
    let requests = stub.requests();
    assert_eq!(
        requests.len(),
        bodies.len(),
        "schema validation moved into the relay"
    );
    for (request, expected) in requests.iter().zip(bodies) {
        let forwarded = request
            .split_once("\r\n\r\n")
            .expect("the signer request has a body")
            .1;
        assert_eq!(forwarded, expected);
    }
}

#[test]
fn the_origin_a_caller_presents_is_the_origin_the_service_sees() {
    let stub = StubService::start();
    stub.on(
        "POST /signer/api/v1/sign/transaction",
        403,
        r#"{"allowed":false,"code":"origin-not-allowed","message":"this caller did not name your origin"}"#,
    )
    // Queued for the request at the end of this test, where the credential was
    // not a bearer token at all.
    .on(
        "POST /signer/api/v1/sign/transaction",
        403,
        r#"{"allowed":false,"code":"missing-token","message":"no bearer token was presented"}"#,
    );
    let server = spawn_custody_server(&stub);
    let http = agent();

    post_sign(
        &http,
        &sign_url(&server),
        Some(CALLER_CREDENTIAL),
        Some("https://relayer.example"),
        KEY_ID,
        "7a7a",
    );
    // The other direction matters as much: an absent `Origin` has to stay absent,
    // because "a server-to-server caller presented no origin" and "a caller
    // presented an empty one" are different answers to §4.3.
    post_sign(
        &http,
        &sign_url(&server),
        Some(CALLER_CREDENTIAL),
        None,
        KEY_ID,
        "7a7a",
    );
    let requests = stub.requests();
    assert!(
        requests[0].contains("origin: https://relayer.example"),
        "{}",
        requests[0]
    );
    assert!(!requests[1].contains("origin:"), "{}", requests[1]);

    // Authorization is one of the signer's exact authentication inputs. The
    // relay forwards even an unsupported scheme unchanged and lets the signer
    // classify it as `missing-token`; it never rewrites it into a bearer value.
    let foreign = into_response(
        http.post(&sign_url(&server))
            .set("authorization", "Basic dXNlcjpwYXNz")
            .set("content-type", "application/json")
            .send_string(
                &serde_json::json!({ "key_id": KEY_ID, "unsigned_hex": "7a7a" }).to_string(),
            ),
    );
    assert_eq!(json_body(foreign)["code"], "missing-token");
    let forwarded = stub.requests().pop().expect("the relay asked");
    assert!(
        forwarded.contains("authorization: basic dxnlcjpwyxnz"),
        "{forwarded}"
    );
    assert!(!forwarded.contains("authorization: bearer"), "{forwarded}");
}

#[test]
fn a_console_with_no_custody_says_so_and_asks_nothing() {
    // The state a workbench starts in when no signer backend is selected for the
    // console. It is not an empty remote vault, and the page must say that
    // explicitly instead of presenting legacy process-wide environment setup.
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);

    let page = into_response(
        http.get(&format!("{base}/signer"))
            .set("cookie", &session)
            .call(),
    );
    assert_eq!(page.status(), 200);
    let text = page.into_string().expect("utf-8 body");
    assert!(text.contains("Custody is not configured"), "{text}");
    assert!(
        text.contains("no signer backend is selected for console"),
        "the page explains why custody actions are unavailable: {text}"
    );
    assert!(
        text.contains("Custody inventory unavailable"),
        "the page distinguishes an unconfigured backend from an empty vault: {text}"
    );
    assert!(!text.contains(neo_nexus::signer_client::URL_ENV), "{text}");
    assert!(
        !text.contains(neo_nexus::signer_client::TOKEN_FILE_ENV),
        "{text}"
    );
    assert!(
        !text.contains(neo_nexus::signer_client::WORKLOAD_KEY_FILE_ENV),
        "{text}"
    );
    assert!(!text.contains("Remote custody connected"), "{text}");

    let response = post_sign(
        &http,
        &sign_url(&server),
        Some(CALLER_CREDENTIAL),
        None,
        KEY_ID,
        "7a7a",
    );
    assert_eq!(response.status(), 503);
    let body = json_body(response);
    assert_eq!(body["code"], "signer-service-unavailable");
    assert_eq!(
        body["message"],
        "the signer service is unavailable; no local signing fallback exists"
    );

    // The controls answer the same way, because they ask the same question: the
    // flash carries the reason there is no vault, rather than a generated key that
    // was never generated.
    let saved = post_form_as(
        &http,
        &session,
        &format!("{base}/signer/keys/generate"),
        "label=treasury&network=testnet",
    );
    assert_eq!(saved.status(), 303);
    let location = saved.header("location").expect("the flash redirect");
    assert!(location.contains("not%20saved"), "{location}");
    assert!(
        location.contains("no%20signer%20backend%20is%20selected%20for%20console"),
        "{location}"
    );
}

#[test]
fn the_control_plane_exposes_no_private_key_input_or_import_route() {
    let stub = StubService::start();
    stub.with_inventory();
    let server = spawn_custody_server(&stub);
    let http = agent();
    let session = signed_in(&http, &server.base_url);

    let page = into_response(
        http.get(&format!("{}/signer", server.base_url))
            .set("cookie", &session)
            .call(),
    )
    .into_string()
    .expect("utf-8 page");
    for forbidden in [
        r#"name="private_key""#,
        r#"name="passphrase""#,
        r#"name="nep2""#,
        "/signer/keys/import",
        "Import raw key",
        "Import NEP-2",
    ] {
        assert!(!page.contains(forbidden), "found {forbidden:?} in {page}");
    }
    let requests_before = stub.requests().len();

    for path in ["/signer/keys/import", "/signer/keys/import-nep2"] {
        let response = into_response(
            http.post(&format!("{}{path}", server.base_url))
                .set("cookie", &session)
                .set("origin", &server.base_url)
                .set("content-type", "application/x-www-form-urlencoded")
                .send_string("private_key=must-not-be-accepted"),
        );
        assert_eq!(response.status(), 405, "{path} still accepts local input");
    }
    assert_eq!(
        stub.requests().len(),
        requests_before,
        "an absent local route still reached custody"
    );
}

#[test]
fn the_control_plane_can_generate_a_chain_bound_neox_key() {
    let stub = StubService::start();
    stub.with_inventory().on(
        "POST /signer/api/v1/keys",
        200,
        r#"{"allowed":true,"key_id":"key-x","label":"NeoX treasury","network":"mainnet","network_magic":860833102,"chain_family":"neox","chain_id":47763,"public_key":"02ab","script_hash":"0x12","address":"0xab","verification_script":"","signing_enabled":true}"#,
    );
    let server = spawn_custody_server(&stub);
    let http = agent();
    let session = signed_in(&http, &server.base_url);

    let page = into_response(
        http.get(&format!("{}/signer", server.base_url))
            .set("cookie", &session)
            .call(),
    )
    .into_string()
    .expect("utf-8 page");
    for field in ["chain_family", "chain_id", "network_magic"] {
        assert!(
            page.contains(&format!(r#"name="{field}""#)),
            "the generation form omitted {field}: {page}"
        );
    }
    assert!(page.contains("neo-n3"), "{page}");
    assert!(page.contains("neox"), "{page}");

    let response = post_form_as(
        &http,
        &session,
        &format!("{}/signer/keys/generate", server.base_url),
        "label=NeoX+treasury&network=mainnet&chain_family=neox&chain_id=47763&network_magic=",
    );
    assert_eq!(response.status(), 303);

    let request = stub
        .requests()
        .into_iter()
        .find(|request| first_line(request) == "POST /signer/api/v1/keys")
        .expect("the generation request reached custody");
    assert!(request.contains(r#""chain_family":"neox""#), "{request}");
    assert!(request.contains(r#""chain_id":47763"#), "{request}");
    assert!(!request.contains("network_magic"), "{request}");
    assert!(
        request.contains(&format!("authorization: bearer {ADMIN_CREDENTIAL}")),
        "{request}"
    );
}

#[test]
fn a_new_caller_token_is_shown_once_and_never_in_a_url() {
    const NEW_TOKEN: &str = "nsk1_fresh_caller_4d19";
    let stub = StubService::start();
    stub.with_inventory();
    stub.on(
        "POST /signer/api/v1/callers",
        200,
        &format!(r#"{{"allowed":true,"caller":{CALLER_ROW},"token":"{NEW_TOKEN}"}}"#),
    );
    let server = spawn_custody_server(&stub);
    let http = agent();
    let session = signed_in(&http, &server.base_url);

    let created = post_form_as(
        &http,
        &session,
        &format!("{}/signer/callers", server.base_url),
        // `keys` is absent, not empty: an unchecked box submits nothing at all,
        // and a repeated field with an empty value is a form no browser sends.
        "label=relayer&grant=any&capability=sign&origins=",
    );
    // The deliberate exception to the flash-redirect idiom this file otherwise
    // asserts: a token carried by `?flash=` lands in browser history, in the
    // access log and in the next `Referer`, so this one control answers 200.
    assert_eq!(created.status(), 200);
    assert_eq!(created.header("location"), None);
    assert_eq!(created.header("cache-control"), Some("no-store"));
    let page = created.into_string().expect("utf-8 body");
    assert!(page.contains(NEW_TOKEN), "{page}");
    assert!(page.contains("shown once"), "{page}");

    // What the form said is what the service was asked, with the console's own
    // admin credential — the one route where substituting the caller's would be
    // the bug (§5.1 puts `admin` plus a whole-vault grant behind a caller record).
    let requests = stub.requests();
    let request = &requests[0];
    assert!(request.contains("POST /signer/api/v1/callers"), "{request}");
    assert!(
        request.contains(&format!("authorization: bearer {ADMIN_CREDENTIAL}")),
        "{request}"
    );
    assert!(request.contains(r#""capabilities":["sign"]"#), "{request}");
    assert!(request.contains(r#""mode":"any""#), "{request}");
    assert!(request.contains(r#""label":"relayer""#), "{request}");

    // The list page carries no copy of it.
    let listing = into_response(
        http.get(&format!("{}/signer", server.base_url))
            .set("cookie", &session)
            .call(),
    );
    assert_eq!(
        listing.header("cache-control"),
        Some("no-store"),
        "signer inventory must never be cached even after the one-time credential is gone"
    );
    let text = listing.into_string().expect("utf-8 body");
    assert!(text.contains("relayer"), "{text}");
    assert!(
        !text.contains(NEW_TOKEN),
        "the token was rendered a second time"
    );

    // Rotation renders the same way, because it mints the same kind of thing.
    const ROTATED: &str = "nsk1_rotated_caller_8e77";
    stub.on(
        &format!("POST /signer/api/v1/callers/{CALLER_ID}/rotate"),
        200,
        &format!(r#"{{"allowed":true,"caller_id":"{CALLER_ID}","token":"{ROTATED}"}}"#),
    );
    let rotated = post_form_as(
        &http,
        &session,
        &format!("{}/signer/callers/{CALLER_ID}/rotate", server.base_url),
        "",
    );
    assert_eq!(rotated.status(), 200);
    assert_eq!(rotated.header("cache-control"), Some("no-store"));
    let page = rotated.into_string().expect("utf-8 body");
    assert!(page.contains(ROTATED), "{page}");
    assert!(
        !page.contains(NEW_TOKEN),
        "a rotation showed the token it just replaced: {page}"
    );
    let request = stub
        .requests()
        .into_iter()
        .find(|request| {
            first_line(request) == format!("POST /signer/api/v1/callers/{CALLER_ID}/rotate")
        })
        .expect("the rotate route reached the service");
    assert!(
        request.contains(&format!("authorization: bearer {ADMIN_CREDENTIAL}")),
        "{request}"
    );
    assert!(
        !request.contains(NEW_TOKEN),
        "a rotation was asked by presenting the token it was replacing: {request}"
    );
}

#[test]
fn a_workload_identity_is_registered_without_secret_ingress_or_token_rotation() {
    let stub = StubService::start();
    stub.on(
        "GET /signer/api/v1/keys",
        200,
        &format!(r#"{{"allowed":true,"keys":[{KEY_ROW}]}}"#),
    )
    .on(
        "GET /signer/api/v1/callers",
        200,
        &format!(r#"{{"allowed":true,"callers":[{WORKLOAD_CALLER_ROW},{FUTURE_CALLER_ROW}]}}"#),
    )
    .on(
        "GET /signer/api/v1/audit",
        200,
        r#"{"allowed":true,"entries":[]}"#,
    )
    .on(
        "POST /signer/api/v1/callers/workload",
        200,
        &format!(r#"{{"allowed":true,"caller":{WORKLOAD_CALLER_ROW}}}"#),
    );
    let server = spawn_custody_server(&stub);
    let http = agent();
    let session = signed_in(&http, &server.base_url);

    let page = into_response(
        http.get(&format!("{}/signer", server.base_url))
            .set("cookie", &session)
            .call(),
    )
    .into_string()
    .expect("utf-8 page");
    for field in ["workload_public_key", "workload_subject"] {
        assert!(
            page.contains(&format!(r#"name="{field}""#)),
            "{field}: {page}"
        );
    }
    assert!(page.contains("workload-ed25519 · relayer-prod"), "{page}");
    assert!(
        !page.contains(&format!("/signer/callers/{WORKLOAD_CALLER_ID}/rotate")),
        "a workload identity has no bearer token to rotate: {page}"
    );
    assert!(
        page.contains("mtls-spiffe identity: no bearer token to rotate"),
        "{page}"
    );
    assert!(
        !page.contains(&format!("/signer/callers/{FUTURE_CALLER_ID}/rotate")),
        "an unknown non-bearer auth mode must fail closed instead of gaining token rotation: {page}"
    );

    let response = post_form_as(
        &http,
        &session,
        &format!("{}/signer/callers/workload", server.base_url),
        &format!(
            "label=production+relayer&grant=only&keys={KEY_ID}&capability=sign&origins=&workload_public_key={}&workload_subject=relayer-prod",
            "aa".repeat(32)
        ),
    );
    assert_eq!(response.status(), 303);
    assert!(
        response
            .header("location")
            .is_some_and(|location| location.contains("workload%20caller")),
        "the public identity registration did not produce the expected flash redirect"
    );

    let request = stub
        .requests()
        .into_iter()
        .find(|request| first_line(request) == "POST /signer/api/v1/callers/workload")
        .expect("the workload caller route reached custody");
    assert!(
        request.contains(r#""workload_public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa""#),
        "{request}"
    );
    assert!(
        request.contains(r#""workload_subject":"relayer-prod""#),
        "{request}"
    );
    for forbidden in ["private_key", "passphrase", "nep2", r#""token":"#] {
        assert!(
            !request.contains(forbidden),
            "{forbidden} leaked into {request}"
        );
    }
}

#[test]
fn the_key_page_represents_the_complete_current_policy_surface() {
    let stub = StubService::start();
    let key_fields = &KEY_ROW[1..KEY_ROW.len() - 1];
    stub.on(
        &format!("GET /signer/api/v1/keys/{KEY_ID}/policy"),
        200,
        &format!(
            r#"{{"allowed":true,{key_fields},"chain_family":"neox","chain_id":47763,"problems":[],"policy":{{"allow_consensus":false,"allow_raw":false,"allow_transfer":true,"allow_contract_call":true,"allow_global_scope":false,"contract_whitelist":[],"contract_blacklist":[],"contract_method_whitelist":[{{"contract":"0xef4073a0f2bacd0bc1d5de799e3b661d633eb9f9","method":"transfer"}}],"contract_method_blacklist":[],"asset_whitelist":[],"asset_blacklist":[],"asset_limits":[{{"asset":"0xef4073a0f2bacd0bc1d5de799e3b661d633eb9f9","max_single_amount":"10","window_limit":{{"seconds":60,"max_amount":"100"}}}}],"transfer_to_whitelist":[],"transfer_to_blacklist":[],"max_single_amount":null,"window_limit":null,"max_signers":2,"max_system_fee":"100000000","max_network_fee":"20000000","max_signatures":{{"seconds":60,"count":5}},"chain_family":"neox","evm_max_gas_price":"30000000000","evm_max_gas_limit":250000,"evm_method_whitelist":["0xa9059cbb"],"evm_method_blacklist":["0x095ea7b3"],"evm_chain_id":47763,"future_daily_request_limit":{{"seconds":86400,"count":7}}}}}}"#
        ),
    )
    .on(
        "GET /signer/api/v1/audit",
        200,
        r#"{"allowed":true,"entries":[]}"#,
    )
    .on(
        "GET /signer/api/v1/callers",
        200,
        r#"{"allowed":true,"callers":[]}"#,
    );
    let server = spawn_custody_server(&stub);
    let http = agent();
    let response = into_response(
        http.get(&format!("{}/signer/keys/{KEY_ID}", server.base_url))
            .set("cookie", &signed_in(&http, &server.base_url))
            .call(),
    );
    assert_eq!(response.status(), 200);
    let page = response.into_string().expect("utf-8 body");
    for field in [
        "contract_method_whitelist",
        "contract_method_blacklist",
        "asset_limits",
        "max_signers",
        "max_system_fee",
        "max_network_fee",
        "signature_window_seconds",
        "signature_window_count",
        "chain_family",
        "evm_max_gas_price",
        "evm_max_gas_limit",
        "evm_method_whitelist",
        "evm_method_blacklist",
        "evm_chain_id",
        "additional_fields",
    ] {
        assert!(
            page.contains(&format!(r#"name="{field}""#)),
            "{field} missing"
        );
    }
    for preserved in [
        "neox · 47763",
        "transfer",
        "30000000000",
        "250000",
        "0xa9059cbb",
        "0x095ea7b3",
        "47763",
        "future_daily_request_limit",
    ] {
        assert!(page.contains(preserved), "{preserved} was not rendered");
    }
    assert!(
        page.contains(r#"<option value="neo-n3">neo-n3</option>"#),
        "the policy form must use the signer's canonical Neo N3 wire value"
    );
    assert!(
        page.contains(r#"<option value="neox" selected>neox</option>"#),
        "the service's current chain family must remain selected"
    );
    assert!(
        !page.contains("neo_n3"),
        "the rejected underscore alias leaked into the form"
    );
}

#[test]
fn the_boundary_reported_back_is_the_one_the_service_stored() {
    let policy_url = format!("POST /signer/api/v1/keys/{KEY_ID}/policy");
    let stub = StubService::start();
    stub.with_inventory();
    // The service says it stored a boundary with transfers closed and adds advice
    // about a shape it judged weaker. The form said transfers were enabled. Only
    // one of those two is what the key will do on the next request, and the page
    // has to report the first.
    stub.on(
        &policy_url,
        200,
        r#"{"allowed":true,"policy":{"allow_consensus":false,"allow_raw":true,"allow_transfer":false,"allow_contract_call":false,"allow_global_scope":false,"asset_whitelist":["0xef4073a0f2bacd0bc1d5de799e3b661d633eb9f9","typo"],"max_single_amount":"1e6"},"problems":[{"code":"transfers-without-recipients","message":"transfers are allowed and the recipient lists are empty"}]}"#,
    );
    let server = spawn_custody_server(&stub);
    let http = agent();
    let session = signed_in(&http, &server.base_url);
    let form = "allow_consensus=disabled&allow_raw=enabled&allow_transfer=enabled&allow_contract_call=disabled\
                &allow_global_scope=disabled&asset_whitelist=0xef4073a0f2bacd0bc1d5de799e3b661d633eb9f9,%20typo\
                &max_single_amount=1e6";

    let saved = post_form_as(
        &http,
        &session,
        &format!("{}/signer/keys/{KEY_ID}/policy", server.base_url),
        form,
    );
    assert_eq!(saved.status(), 303, "the boundary did not save");
    let location = saved.header("location").expect("the flash redirect");
    let redirect = format!("/signer/keys/{KEY_ID}?flash=");
    assert!(location.starts_with(redirect.as_str()), "{location}");
    assert!(location.contains("raw%20allowed"), "{location}");
    assert!(location.contains("transfers%20closed"), "{location}");
    assert!(!location.contains("transfers%20allowed"), "{location}");
    // The advice travels with the boundary, counted in the flash and listed in
    // full on the page the redirect lands on.
    assert!(location.contains("weaker"), "{location}");

    // Unvalidated entries and an amount the console could not parse reached the
    // service as written. That is the whole point of §4.2 living on the service
    // side: a form that dropped `typo` would save a boundary the operator never
    // wrote, and one that rejected `1e6` locally would be a second set of rules
    // about what an amount is.
    let request = stub
        .requests()
        .into_iter()
        .find(|request| first_line(request) == policy_url)
        .expect("the policy save reached the service");
    assert!(request.contains(r#""typo""#), "{request}");
    assert!(
        request.contains(r#""max_single_amount":"1e6""#),
        "{request}"
    );
    assert!(request.contains(r#""allow_transfer":true"#), "{request}");
    assert!(request.contains(r#""allow_raw":true"#), "{request}");
    assert_eq!(
        stub.asked(&policy_url),
        1,
        "the console asked the service twice about one save"
    );

    // What stays here is the rule about the *form*: half a rolling window is not
    // a boundary with a hole in it, it is a boundary that does nothing, and the
    // operator learns which end was missing without the service being asked to
    // notice.
    let half = post_form_as(
        &http,
        &session,
        &format!("{}/signer/keys/{KEY_ID}/policy", server.base_url),
        "allow_consensus=disabled&allow_raw=disabled&allow_transfer=enabled&allow_contract_call=disabled\
         &allow_global_scope=disabled&window_seconds=3600",
    );
    assert_eq!(half.status(), 303);
    let location = half.header("location").expect("the flash redirect");
    assert!(location.contains("not%20saved"), "{location}");
    assert_eq!(
        stub.asked(&policy_url),
        1,
        "a form this process could not finish still reached the vault"
    );
}

/// The logs page requires authentication and supports clearing all .log files
/// from the workspace logs directory via POST with explicit confirmation.
#[test]
fn logs_page_requires_authentication() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;

    // Anonymous request should redirect to login
    let response = into_response(http.get(&format!("{base}/logs")).call());
    assert_eq!(response.status(), 303);
    assert_eq!(response.header("location"), Some("/login"));

    // Authenticated request should render the page
    let login = post_form(&http, &format!("{base}/login"), &format!("token={TOKEN}"));
    let session = cookie_value(&login).expect("session cookie set");

    let page = into_response(
        http.get(&format!("{base}/logs"))
            .set("cookie", &session)
            .call(),
    );
    assert_eq!(page.status(), 200);
    let body = page.into_string().expect("logs page body");
    assert!(body.contains("<h1>Logs</h1>"));
}

/// Clearing logs via POST endpoint clears .log files and records an event.
#[test]
fn clear_logs_cleared_log_files_and_records_event() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let login = post_form(&http, &format!("{base}/login"), &format!("token={TOKEN}"));
    let session = cookie_value(&login).expect("session cookie set");

    // Create some log files in the workspace logs directory
    let logs_dir = server.state.workspace_child_dir("logs");
    std::fs::create_dir_all(&logs_dir).expect("create logs dir");

    let log_file1 = logs_dir.join("node.log");
    let log_file2 = logs_dir.join("error.LOG");
    let non_log_file = logs_dir.join("config.txt");

    std::fs::write(&log_file1, "some log content").expect("write log1");
    std::fs::write(&log_file2, "another log").expect("write log2");
    std::fs::write(&non_log_file, "not a log").expect("write non-log");

    // Verify files exist before clearing
    assert!(log_file1.exists());
    assert!(log_file2.exists());
    assert!(non_log_file.exists());

    // POST to /logs (with node context to preserve selection)
    let response = post_form_as(&http, &session, &format!("{base}/logs"), "node=");
    assert_eq!(response.status(), 303);

    let location = response.header("location").expect("redirect location");
    assert!(location.starts_with("/logs?flash="));
    assert!(location.contains("cleared+2+log+file(s)"));

    // Verify log files were deleted but non-log files remain
    assert!(!log_file1.exists());
    assert!(!log_file2.exists());
    assert!(non_log_file.exists());

    // Verify LogCleared event was recorded
    let repository = Repository::open(&server.db_path).expect("reopen workspace");
    let events = repository
        .list_events(neo_nexus::core::operations::RuntimeEventFilter::new(
            None, "", 200,
        ))
        .expect("events");

    let has_log_cleared = events
        .iter()
        .any(|event| event.kind.to_string() == "log-cleared");
    assert!(has_log_cleared, "LogCleared event should be recorded");
}

/// Clear logs without authentication is rejected.
#[test]
fn clear_logs_rejected_without_session() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;

    // Try to POST without authentication
    let response = into_response(
        http.post(&format!("{base}/logs"))
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string("node=")
            .call(),
    );

    // Should redirect to login instead of processing the request
    assert_eq!(response.status(), 303);
    assert_eq!(response.header("location"), Some("/login"));
}
