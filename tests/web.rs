//! End-to-end web workbench coverage: a real server on an ephemeral port, a
//! real workspace database, and plain HTTP through the library's own `ureq`
//! dependency. The suite pins the auth boundary, the JSON API, the page
//! render, and the lifecycle control path — the same pipeline the CLI uses.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use axum::serve;
use neo_nexus::{
    repository::Repository,
    types::{Network, NewNode, NodeType, StorageEngine},
    web::{auth::AuthStore, html, nav, router::build_router, WebState},
};
use ureq::AgentBuilder;

const TOKEN: &str = "web-suite-token";

struct Server {
    base_url: String,
    db_path: PathBuf,
    // The runtime owns the accept loop; dropping it stops the server, so it has
    // to outlive every request the test makes. Declared before `_home` so the
    // server is shut down before the temp workspace disappears.
    _runtime: tokio::runtime::Runtime,
    // Keep the tempdir alive for the whole test.
    _home: tempfile::TempDir,
}

fn spawn_server() -> Server {
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
        AuthStore::from_token(TOKEN),
    );
    let address = runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("ephemeral bind");
        let address = listener.local_addr().expect("bound address");
        tokio::spawn(async move {
            serve(listener, build_router(state))
                .await
                .expect("server task");
        });
        address
    });
    Server {
        base_url: format!("http://{address}"),
        db_path,
        _runtime: runtime,
        _home: home,
    }
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
    into_response(
        agent
            .post(url)
            .set("cookie", session)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string(body),
    )
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
    let body = json_body(response);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["application"], "NeoNexus");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
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
    let session = cookie_value(&accepted).expect("session cookie set");
    assert!(session.starts_with("neonexus_session="));

    let home = into_response(http.get(base).set("cookie", &session).call());
    assert_eq!(home.status(), 200);
    assert!(home.into_string().unwrap().contains("Fleet overview"));
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
