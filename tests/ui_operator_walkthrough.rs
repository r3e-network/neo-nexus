//! Operator walkthrough for the v3.2.0 UI density feature: a real server on an
//! ephemeral port, a real workspace database, and plain HTTP through the
//! library's own `ureq` dependency — the same harness `tests/web.rs` uses.
//!
//! The feature promises three things a browser operator can see: the shell
//! carries the comfortable density by default, switching to compact both
//! re-renders the fleet as the single-line anatomy *and* survives the next page
//! load, and the sidebar/header chrome never moves between the two modes. Each
//! is driven through the real settings POST rather than by fabricating stored
//! state, so the test exercises the same persistence path production does.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use axum::serve;
use neo_nexus::{
    repository::Repository,
    types::{Network, NewNode, NodeType, StorageEngine},
    web::{
        auth::{AuthStore, WebSecurity},
        router::build_router,
        Custody, WebState,
    },
};
use ureq::AgentBuilder;

const TOKEN: &str = "density-suite-token-4c1f8b2a9d7e46f3ab05c1d2e3f40567";

struct Server {
    base_url: String,
    db_path: PathBuf,
    // The runtime owns the accept loop; dropping it stops the server, so it has
    // to outlive every request. Declared before `_home` so the server is shut
    // down before the temp workspace disappears.
    _runtime: tokio::runtime::Runtime,
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
        AuthStore::from_token(TOKEN).expect("strong test token"),
        WebSecurity::loopback_http(),
    )
    .expect("valid state configuration")
    .with_custody(Custody::unconfigured());
    // `build_router` consumes its state, so hand it a clone: WebState is shared
    // by design and cheap to clone.
    let router_state = state.clone();
    let address = runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("ephemeral bind");
        let address = listener.local_addr().expect("bound address");
        tokio::spawn(async move {
            serve(
                listener,
                build_router(router_state).into_make_service_with_connect_info::<SocketAddr>(),
            )
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

/// ureq reports every status `>= 400` as `Error::Status`; fold both arms back
/// into a response so assertions can test the status code itself.
fn into_response(result: Result<ureq::Response, ureq::Error>) -> ureq::Response {
    let response = match result {
        Ok(response) => Some(response),
        Err(ureq::Error::Status(_, response)) => Some(response),
        Err(_) => None,
    };
    response.expect("request reaches the workbench server")
}

fn post_form(agent: &ureq::Agent, url: &str, body: &str) -> ureq::Response {
    into_response(
        agent
            .post(url)
            .set("content-type", "application/x-www-form-urlencoded")
            .send_string(body),
    )
}

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

fn signed_in(http: &ureq::Agent, base: &str) -> String {
    let login = post_form(http, &format!("{base}/login"), &format!("token={TOKEN}"));
    cookie_value(&login).expect("session cookie set")
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

/// GET a protected page as the signed-in operator and return the rendered body.
fn get_page(http: &ureq::Agent, session: &str, url: &str) -> String {
    let response = into_response(http.get(url).set("cookie", session).call());
    assert_eq!(response.status(), 200, "{url} must render when signed in");
    response.into_string().expect("page body")
}

/// Persist the compact density through the real settings POST, exactly as the
/// Appearance form does (`ui_density` field, normalised through `DensityMode`).
fn set_density(http: &ureq::Agent, session: &str, base: &str, choice: &str) {
    let saved = post_form_as(
        http,
        session,
        &format!("{base}/settings/density"),
        &format!("ui_density={choice}"),
    );
    assert_eq!(saved.status(), 303, "a density save redirects back");
    let location = saved.header("location").expect("redirect after save");
    assert!(
        location.starts_with("/settings"),
        "density save returns to settings: {location}"
    );
}

/// Isolate the density-invariant chrome — the mobile header and the sidebar —
/// so a test can assert it is byte-identical across modes. Both blocks come
/// from `nav::render(active)`, which never reads the density, so any drift here
/// would mean the density class had leaked out of the page body.
fn chrome(body: &str) -> String {
    let header_start = body
        .find(r#"<header class="mobile-nav">"#)
        .expect("mobile header present");
    let sidebar_end = body.find("</aside>").expect("sidebar present") + "</aside>".len();
    body[header_start..sidebar_end].to_string()
}

#[test]
fn density_comfortable_rendering() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);
    create_node(&server.db_path, "comfortable-node", 41332);

    // Density is unset, so the shell falls back to the comfortable default.
    let nodes = get_page(&http, &session, &format!("{base}/nodes"));
    assert!(
        nodes.contains(r#"<body class="density-comfortable">"#),
        "the default shell must carry the comfortable class"
    );
    // The stylesheet always defines both `.density-*` modifiers; what matters
    // is that the shell itself does not carry the compact one.
    assert!(
        !nodes.contains(r#"<body class="density-compact">"#),
        "the shell must not carry the compact class when density is unset"
    );
    // The comfortable fleet is the multi-column table, not the compact anatomy.
    assert!(
        !nodes.contains(r#"class="node-line""#),
        "comfortable renders the wide table, not the single-line row"
    );
    // The comfortable fleet is the wide inventory table. These are the node's
    // genuinely independent axes — the process and what the chain says — the
    // same columns `tests/web.rs` asserts, not the old fused "Status Check"
    // badge this console retired.
    assert!(
        nodes.contains(">Process</th>"),
        "the comfortable table keeps its own columns"
    );
    assert!(nodes.contains(">Chain health</th>"));
    assert!(nodes.contains(">Height</th>"));

    // The home page shares the shell and also renders comfortable by default.
    let home = get_page(&http, &session, &format!("{base}/"));
    assert!(home.contains(r#"<body class="density-comfortable">"#));
}

#[test]
fn density_compact_rendering() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);
    create_node(&server.db_path, "compact-node", 42332);

    set_density(&http, &session, base, "compact");

    let nodes = get_page(&http, &session, &format!("{base}/nodes"));
    assert!(
        nodes.contains(r#"<body class="density-compact">"#),
        "the shell must carry the compact class once it is stored"
    );
    // The compact single-line anatomy: a status dot leading the row, the node
    // name link, and the RPC port chip — the markup that trades columns for
    // density.
    assert!(
        nodes.contains(r#"class="node-line""#),
        "compact must render the single-line node anatomy"
    );
    assert!(
        nodes.contains(r#"class="status-dot"#),
        "the compact row leads with a status dot"
    );
    assert!(
        nodes.contains(r#"class="node-name""#),
        "the compact row keeps a named link to the node"
    );
    assert!(
        nodes.contains("RPC 42332"),
        "the compact row still shows the RPC port"
    );
}

#[test]
fn chrome_invariance_across_modes() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);
    create_node(&server.db_path, "chrome-node", 43332);

    // Comfortable first, then compact through the same settings path.
    let comfortable = get_page(&http, &session, &format!("{base}/nodes"));
    set_density(&http, &session, base, "compact");
    let compact = get_page(&http, &session, &format!("{base}/nodes"));

    // The one thing that must differ is the body density class.
    assert!(comfortable.contains(r#"<body class="density-comfortable">"#));
    assert!(compact.contains(r#"<body class="density-compact">"#));

    // The chrome must be present in both...
    let comfortable_chrome = chrome(&comfortable);
    let compact_chrome = chrome(&compact);
    assert!(
        comfortable_chrome.contains(r#"<nav class="sidebar-nav" aria-label="Primary navigation">"#),
        "the sidebar chrome must render"
    );
    assert!(
        comfortable_chrome.contains("nav-item"),
        "the sidebar must carry its navigation items"
    );
    // ...and byte-identical: the density class scopes the body, never the shell.
    assert_eq!(
        comfortable_chrome, compact_chrome,
        "sidebar/header chrome must be density-invariant"
    );
}

#[test]
fn density_preference_persists_across_page_loads() {
    let server = spawn_server();
    let http = agent();
    let base = &server.base_url;
    let session = signed_in(&http, base);
    create_node(&server.db_path, "persist-node", 44332);

    // Starts comfortable.
    let before = get_page(&http, &session, &format!("{base}/nodes"));
    assert!(before.contains(r#"<body class="density-comfortable">"#));

    // A POST through the settings page flips it, and the very next page load —
    // a fresh request against the stored preference — reflects the change.
    set_density(&http, &session, base, "compact");
    let after_compact = get_page(&http, &session, &format!("{base}/nodes"));
    assert!(
        after_compact.contains(r#"<body class="density-compact">"#),
        "the compact choice must persist into the next load"
    );

    // It also confirms in the database, not just in the rendered class.
    let stored = Repository::open(&server.db_path)
        .expect("reopen workspace")
        .load_app_ui_density()
        .expect("density read")
        .expect("a density was stored");
    assert_eq!(stored, "compact", "the compact key must be persisted");

    // Switching back is just as durable, so the preference is a real toggle and
    // not a one-way latch.
    set_density(&http, &session, base, "comfortable");
    let after_comfortable = get_page(&http, &session, &format!("{base}/nodes"));
    assert!(
        after_comfortable.contains(r#"<body class="density-comfortable">"#),
        "switching back to comfortable must persist too"
    );
}
