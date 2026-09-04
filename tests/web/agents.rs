use super::*;

#[test]
fn companion_forms_require_auth_and_allow_reviewing_stopped_profiles() {
    let server = spawn_server();
    let http = agent();
    let endpoint = format!("{}/agents/save", server.base_url);
    let denied = post_form(&http, &endpoint, "name=no-session");
    assert_eq!(denied.header("location"), Some("/login"));
    let login = post_form(
        &http,
        &format!("{}/login", server.base_url),
        &format!("token={TOKEN}"),
    );
    let cookie = cookie_value(&login).unwrap();
    let body = [
        ("id", "test-agent".to_string()),
        ("name", "<node companion>".into()),
        ("kind", "sidecar".into()),
        ("version", "1.2.3".into()),
        (
            "binary_path",
            std::env::current_exe().unwrap().display().to_string(),
        ),
        ("working_dir", server._home.path().display().to_string()),
        ("args", "[]".into()),
    ]
    .into_iter()
    .map(|(key, value)| format!("{key}={}", html::urlencoding_lite(&value)))
    .collect::<Vec<_>>()
    .join("&");
    let saved = post_form_as(&http, &cookie, &endpoint, &body);
    assert_eq!(saved.status(), 303);
    assert!(saved.header("location").unwrap().contains("saved"));
    let page = http
        .get(&format!("{}/agents?edit=test-agent", server.base_url))
        .set("cookie", &cookie)
        .call()
        .unwrap()
        .into_string()
        .unwrap();
    assert!(page.contains("&lt;node companion&gt;"));
    assert!(page.contains("1.2.3"));
    assert!(page.contains("HERMES_HOME"));
    assert_eq!(server.state.repository.list_agents().unwrap().len(), 1);
    let removed = post_form_as(
        &http,
        &cookie,
        &format!("{}/agents/test-agent/delete", server.base_url),
        "",
    );
    assert_eq!(removed.status(), 303);
    assert!(server.state.repository.list_agents().unwrap().is_empty());
}
