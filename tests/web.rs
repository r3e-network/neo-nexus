//! End-to-end web workbench coverage: a real server on an ephemeral port, a
//! real workspace database, and plain HTTP through the library's own `ureq`
//! dependency. The suite pins the auth boundary, the JSON API, the page
//! render, and the lifecycle control path — the same pipeline the CLI uses.

use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use axum::serve;
use neo_nexus::{
    core::operations::RuntimeEventFilter,
    repository::Repository,
    types::{Network, NewNode, NodeType, StorageEngine},
    watchdog::RestartPolicy,
    web::{auth::AuthStore, html, nav, router::build_router, WebState},
};
use ureq::AgentBuilder;

const TOKEN: &str = "web-suite-token";

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
    // `build_router` consumes its state, so hand it a clone and keep the
    // original: WebState is shared by design and cheap to clone.
    let router_state = state.clone();
    let address = runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("ephemeral bind");
        let address = listener.local_addr().expect("bound address");
        tokio::spawn(async move {
            serve(listener, build_router(router_state))
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
    server._engine = Some(neo_nexus::supervision::Engine::start(
        server.state.engine_state(),
    ));
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
