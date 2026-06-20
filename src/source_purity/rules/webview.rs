pub(super) fn is_webview_cargo_package_name(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    normalized == "tauri"
        || normalized.starts_with("tauri-")
        || normalized == "wry"
        || normalized == "web-view"
        || normalized == "webview"
        || normalized.starts_with("webview2-com")
        || normalized.starts_with("webkit2gtk")
        || normalized.starts_with("javascriptcore-rs")
        || normalized.starts_with("objc2-web-kit")
}
