use super::*;
use serde_json::json;
use std::path::PathBuf;

fn profile(home: &std::path::Path) -> AgentProfile {
    AgentProfile {
        id: "hermes-test".into(),
        name: "Hermes test".into(),
        kind: AgentKind::Hermes,
        node_id: None,
        version: "0.21.0".into(),
        binary_path: std::env::current_exe().unwrap(),
        working_dir: home.into(),
        args: vec![],
        config_path: Some(home.join("config.yaml")),
        health_url: None,
        auto_restart: false,
        binary_sha256: String::new(),
        config_sha256: None,
    }
}

#[test]
fn stop_command_is_scoped_and_has_no_shell_or_force_arguments() {
    let dir = tempfile::tempdir().unwrap();
    let profile = profile(dir.path());
    let command = stop_command(&profile).unwrap();
    assert_eq!(command.get_program(), profile.binary_path.as_os_str());
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        ["-m", "hermes_cli.main", "gateway", "stop"]
    );
    let env = command
        .get_envs()
        .map(|(key, value)| {
            (
                key.to_string_lossy().into_owned(),
                value.unwrap().to_string_lossy().into_owned(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(PathBuf::from(&env["HERMES_HOME"]), dir.path());
    assert_eq!(env["HERMES_GATEWAY_EXTERNAL_SUPERVISOR"], "1");
}

#[test]
fn status_requires_the_expected_pid_a_fresh_timestamp_and_a_valid_state() {
    let dir = tempfile::tempdir().unwrap();
    let profile = profile(dir.path());
    let path = dir.path().join("gateway_state.json");
    let timestamp = parse_utc_timestamp("2026-09-05T01:02:03+00:00").unwrap();
    assert_eq!(runtime_health(&profile, 42, timestamp).unwrap(), None);
    let mut status = json!({"pid":42,"gateway_state":"running","updated_at":"2026-09-05T01:02:03.123456+00:00","platforms":{}});
    let write =
        |status: &Value| std::fs::write(&path, serde_json::to_vec(status).unwrap()).unwrap();
    write(&status);
    assert_eq!(runtime_health(&profile, 42, timestamp).unwrap(), Some(true));
    assert_eq!(runtime_health(&profile, 43, timestamp).unwrap(), None);
    assert_eq!(runtime_health(&profile, 42, timestamp + 121).unwrap(), None);
    assert_eq!(runtime_health(&profile, 42, timestamp - 31).unwrap(), None);
    status["gateway_state"] = json!("startup_failed");
    write(&status);
    assert_eq!(
        runtime_health(&profile, 42, timestamp).unwrap(),
        Some(false)
    );
    status["gateway_state"] = json!("draining");
    write(&status);
    assert_eq!(runtime_health(&profile, 42, timestamp).unwrap(), None);
    status["gateway_state"] = json!("running");
    status["hermes_home"] = json!(dir.path().join("other-profile"));
    write(&status);
    assert_eq!(runtime_health(&profile, 42, timestamp).unwrap(), None);
}

#[test]
fn current_adapter_and_session_store_failures_degrade_health() {
    let dir = tempfile::tempdir().unwrap();
    let profile = profile(dir.path());
    let timestamp = parse_utc_timestamp("2026-09-05T01:02:03Z").unwrap();
    let path = dir.path().join("gateway_state.json");
    let mut status = json!({"pid":42,"gateway_state":"running","updated_at":"2026-09-05T01:02:03Z",
        "platforms":{"telegram":{"writer_pid":42,"state":"connected","needs_attention":false}}});
    let write =
        |status: &Value| std::fs::write(&path, serde_json::to_vec(status).unwrap()).unwrap();
    write(&status);
    assert_eq!(runtime_health(&profile, 42, timestamp).unwrap(), Some(true));
    status["platforms"]["telegram"]["needs_attention"] = json!(true);
    write(&status);
    assert_eq!(
        runtime_health(&profile, 42, timestamp).unwrap(),
        Some(false)
    );
    status["platforms"]["telegram"]["writer_pid"] = json!(99);
    write(&status);
    assert_eq!(runtime_health(&profile, 42, timestamp).unwrap(), Some(true));
    status["platforms"]["telegram"]["writer_pid"] = json!(42);
    status["platforms"]["telegram"]["writer_start_time"] = json!(99);
    status["start_time"] = json!(100);
    write(&status);
    assert_eq!(runtime_health(&profile, 42, timestamp).unwrap(), Some(true));
    status["session_store"] = json!({"status":"unavailable"});
    write(&status);
    assert_eq!(
        runtime_health(&profile, 42, timestamp).unwrap(),
        Some(false)
    );
}

#[test]
fn bad_status_data_is_bounded_and_errors_do_not_echo_credentials() {
    let dir = tempfile::tempdir().unwrap();
    let profile = profile(dir.path());
    let path = dir.path().join("gateway_state.json");
    std::fs::write(&path, "not-json-private-credential").unwrap();
    assert!(!runtime_health(&profile, 42, 0)
        .unwrap_err()
        .to_string()
        .contains("private-credential"));
    std::fs::write(&path, vec![b' '; MAX_STATUS_BYTES as usize + 1]).unwrap();
    assert!(runtime_health(&profile, 42, 0).is_err());
}

#[test]
fn timestamps_handle_leap_days_utc_suffixes_and_reject_invalid_dates() {
    assert_eq!(parse_utc_timestamp("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(
        parse_utc_timestamp("2000-02-29T00:00:00+00:00"),
        Some(951782400)
    );
    assert_eq!(
        parse_utc_timestamp("2026-09-05T01:02:03.9Z"),
        parse_utc_timestamp("2026-09-05T01:02:03+00:00")
    );
    for text in [
        "2025-02-29T00:00:00Z",
        "2026-01-01T24:00:00Z",
        "2026-01-01T00:00:00+08:00",
        "2026-01-01T00:00:00.Z",
    ] {
        assert_eq!(parse_utc_timestamp(text), None, "{text}");
    }
}

fn wait_witness() -> Child {
    let mut command = if cfg!(windows) {
        let mut command = Command::new(r"C:\Windows\System32\ping.exe");
        command.args(["-n", "60", "127.0.0.1"]);
        command
    } else {
        let mut command = Command::new("/bin/sleep");
        command.arg("60");
        command
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command.spawn().unwrap()
}

#[test]
fn timeout_reaps_only_the_helper_and_reports_the_target_still_alive() {
    let mut helper = wait_witness();
    assert!(!wait_for_stop(&mut helper, std::process::id(), Duration::from_millis(10)).unwrap());
    assert!(helper.try_wait().unwrap().is_some());
    assert!(process_is_live(std::process::id()));
}

#[test]
fn an_already_gone_target_reaps_the_helper_and_reports_completion() {
    let mut helper = wait_witness();
    assert!(wait_for_stop(&mut helper, 4_000_000, Duration::from_secs(1)).unwrap());
    assert!(helper.try_wait().unwrap().is_some());
}
