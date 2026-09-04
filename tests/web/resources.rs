use super::*;

#[test]
fn resource_policy_requires_auth_validates_thresholds_and_appears_on_monitor() {
    let server = spawn_server();
    let http = agent();
    let url = format!("{}/settings/resources", server.base_url);
    let body="enabled=Enabled&interval_seconds=30&disk_warning_mib=2048&disk_critical_mib=4096&memory_warning_percent=10&memory_critical_percent=5";
    assert_eq!(
        post_form(&http, &url, body).header("location"),
        Some("/login")
    );
    let cookie = signed_in(&http, &server.base_url);
    let invalid = post_form_as(&http, &cookie, &url, body);
    assert!(invalid.header("location").unwrap().contains("warning"));
    assert_eq!(
        server
            .state
            .repository
            .load_resource_policy()
            .unwrap()
            .disk_warning_mib,
        5120
    );
    let saved = post_form_as(
        &http,
        &cookie,
        &url,
        &body.replace("disk_critical_mib=4096", "disk_critical_mib=1024"),
    );
    assert!(saved.header("location").unwrap().contains("saved"));
    let page = http
        .get(&format!("{}/monitor", server.base_url))
        .set("cookie", &cookie)
        .call()
        .unwrap()
        .into_string()
        .unwrap();
    assert!(page.contains("Storage and memory"));
    assert!(page.contains("No fresh resource sample"));
    let metrics = http
        .get(&format!("{}/api/metrics-prometheus", server.base_url))
        .set("cookie", &cookie)
        .call()
        .unwrap()
        .into_string()
        .unwrap();
    assert!(metrics.contains("neonexus_resource_sample_fresh 0"));
}

#[test]
fn running_engine_collects_storage_without_a_page_request() {
    let server = spawn_supervised_server();
    assert!(wait_until(Duration::from_secs(6), || server
        .state
        .repository
        .latest_resource_report()
        .unwrap()
        .is_some()));
    let report = server
        .state
        .repository
        .latest_resource_report()
        .unwrap()
        .unwrap();
    assert!(!report.sample_failed);
    assert!(report
        .readings
        .iter()
        .any(|reading| reading.id.starts_with("disk:") && reading.capacity_bytes.is_some()));
}
