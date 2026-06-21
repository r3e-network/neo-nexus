use super::SourcePurityChecker;

#[test]
fn source_purity_accepts_rust_native_tree_and_skips_build_outputs() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n")?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    std::fs::write(
        temp_dir.path().join("src").join("lib.rs"),
        "pub fn ok() {}\n",
    )?;
    std::fs::create_dir_all(temp_dir.path().join("docs"))?;
    std::fs::write(temp_dir.path().join("docs").join("example.json"), "{}\n")?;
    std::fs::create_dir_all(temp_dir.path().join("target"))?;
    std::fs::write(temp_dir.path().join("target").join("package.json"), "{}\n")?;

    let report = SourcePurityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(report.is_success(), "{}", report.to_cli_text());
    assert_eq!(report.status, "pure-rust");
    assert!(report
        .skipped_directories
        .iter()
        .any(|path| path == "target"));
    assert_eq!(report.disallowed_count, 0);

    Ok(())
}

#[test]
fn source_purity_rejects_frontend_and_node_artifacts() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n")?;
    std::fs::write(temp_dir.path().join("package.json"), "{}\n")?;
    std::fs::create_dir_all(temp_dir.path().join("web").join("src"))?;
    std::fs::write(temp_dir.path().join("web").join("src").join("App.tsx"), "")?;
    std::fs::create_dir_all(temp_dir.path().join("src-tauri").join("src"))?;
    std::fs::write(
        temp_dir
            .path()
            .join("src-tauri")
            .join("src")
            .join("main.rs"),
        "fn main() {}\n",
    )?;
    std::fs::write(temp_dir.path().join("tauri.conf.json"), "{}\n")?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    std::fs::write(temp_dir.path().join("src").join("legacy.js"), "")?;

    let report = SourcePurityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(!report.is_success());
    assert_eq!(report.status, "failed");
    assert!(report
        .disallowed_entries
        .iter()
        .any(|finding| finding.path == "package.json"
            && finding.category == "node-toolchain-manifest"));
    assert!(report
        .disallowed_entries
        .iter()
        .any(|finding| finding.path == "web" && finding.category == "frontend-directory"));
    assert!(report
        .disallowed_entries
        .iter()
        .any(|finding| finding.path == "src-tauri"
            && finding.category == "webview-application-directory"));
    assert!(report
        .disallowed_entries
        .iter()
        .any(|finding| finding.path == "tauri.conf.json"
            && finding.category == "webview-application-config"));
    assert!(report.disallowed_entries.iter().any(
        |finding| finding.path == "src/legacy.js" && finding.category == "frontend-source-file"
    ));

    Ok(())
}

#[test]
fn source_purity_rejects_container_and_web_server_deployment_artifacts() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n")?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    std::fs::write(
        temp_dir.path().join("src").join("lib.rs"),
        "pub fn ok() {}\n",
    )?;
    std::fs::write(temp_dir.path().join("Dockerfile"), "FROM node:22\n")?;
    std::fs::write(temp_dir.path().join("docker-compose.yml"), "services: {}\n")?;
    std::fs::create_dir_all(temp_dir.path().join("docs"))?;
    std::fs::write(
        temp_dir.path().join("docs").join("nginx-example.conf"),
        "server {}\n",
    )?;
    std::fs::create_dir_all(temp_dir.path().join("scripts"))?;
    std::fs::write(
        temp_dir.path().join("scripts").join("setup-nginx.sh"),
        "#!/bin/sh\n",
    )?;

    let report = SourcePurityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(!report.is_success());
    assert_eq!(report.status, "failed");
    assert!(report
        .disallowed_entries
        .iter()
        .any(|finding| finding.path == "Dockerfile"
            && finding.category == "container-deployment-artifact"));
    assert!(report
        .disallowed_entries
        .iter()
        .any(|finding| finding.path == "docker-compose.yml"
            && finding.category == "container-deployment-artifact"));
    assert!(report
        .disallowed_entries
        .iter()
        .any(|finding| finding.path == "docs/nginx-example.conf"
            && finding.category == "web-server-deployment-artifact"));
    assert!(report
        .disallowed_entries
        .iter()
        .any(|finding| finding.path == "scripts/setup-nginx.sh"
            && finding.category == "web-server-deployment-artifact"));

    Ok(())
}

#[test]
fn source_purity_rejects_webview_cargo_dependencies() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[package]
name = "native-boundary"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "0.33"
tauri = "2"

[dev-dependencies]
web-view = "0.7"
"#,
    )?;
    std::fs::write(
        temp_dir.path().join("Cargo.lock"),
        r#"
version = 4

[[package]]
name = "native-boundary"
version = "0.1.0"

[[package]]
name = "wry"
version = "0.52.0"
"#,
    )?;
    std::fs::create_dir_all(temp_dir.path().join("src"))?;
    std::fs::write(
        temp_dir.path().join("src").join("lib.rs"),
        "pub fn ok() {}\n",
    )?;

    let report = SourcePurityChecker::check_at(temp_dir.path(), 1_800_000_000)?;

    assert!(!report.is_success());
    assert_eq!(report.status, "failed");
    assert!(report.disallowed_entries.iter().any(|finding| {
        finding.path == "Cargo.toml"
            && finding.category == "webview-cargo-dependency"
            && finding.message.contains("tauri")
    }));
    assert!(report.disallowed_entries.iter().any(|finding| {
        finding.path == "Cargo.toml"
            && finding.category == "webview-cargo-dependency"
            && finding.message.contains("web-view")
    }));
    assert!(report.disallowed_entries.iter().any(|finding| {
        finding.path == "Cargo.lock"
            && finding.category == "webview-cargo-lock-package"
            && finding.message.contains("wry")
    }));

    Ok(())
}
