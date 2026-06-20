use super::super::*;

#[test]
fn native_ui_audit_cli_reports_fixed_panel_application_shell() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    write_native_ui_fixture(temp_dir.path(), false)?;

    let root_arg = temp_dir.path().display().to_string();
    let action = action_from_args(["neo-nexus", "--native-ui-audit", &root_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected native UI audit action");
    };

    assert_eq!(exit_code, 0);
    assert!(text.contains("native-ui-audit: native"));
    assert!(text.contains("required: 12/12"));
    assert!(text.contains("forbidden: 0"));
    assert!(text.contains("finding: none"));
    Ok(())
}

#[test]
fn native_ui_audit_json_cli_rejects_webview_or_scrolling_shell() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    write_native_ui_fixture(temp_dir.path(), true)?;

    let root_arg = temp_dir.path().display().to_string();
    let action = action_from_args(["neo-nexus", "--native-ui-audit-json", &root_arg])?;
    let CliAction::PrintWithExitCode { text, exit_code } = action else {
        anyhow::bail!("expected native UI audit JSON action");
    };

    assert_eq!(exit_code, 1);
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "failed");
    assert_eq!(value["success"], false);
    assert_eq!(value["report"]["required_passed"], 12);
    assert_eq!(value["report"]["required_total"], 12);
    assert!(value["report"]["findings"]
        .as_array()
        .context("missing native UI audit findings")?
        .iter()
        .any(|finding| finding["category"] == "forbidden-ui-marker"
            && finding["marker"] == scroll_area_marker()));
    assert!(value["report"]["findings"]
        .as_array()
        .context("missing native UI audit findings")?
        .iter()
        .any(|finding| finding["category"] == "forbidden-ui-marker"
            && finding["marker"] == "WebView"));
    Ok(())
}

fn write_native_ui_fixture(root: &Path, include_forbidden_markers: bool) -> Result<()> {
    std::fs::create_dir_all(root.join("src").join("app").join("views"))?;
    std::fs::write(
        root.join("Cargo.toml"),
        r#"
[package]
name = "native-fixture"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "0.33"
egui = "0.33"
"#,
    )?;
    std::fs::write(
        root.join("src").join("main.rs"),
        r#"
fn main() {
let options = eframe::NativeOptions {
    viewport: eframe::egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 820.0])
        .with_min_inner_size([1280.0, 760.0]),
    ..Default::default()
};
eframe::run_native("Fixture", options, Box::new(|_| Ok(Box::new(App))));
}
"#,
    )?;
    std::fs::write(
        root.join("src").join("app.rs"),
        r#"
pub struct App;

impl eframe::App for App {
fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("application_header").show(context, |_| {});
    egui::TopBottomPanel::bottom("status_bar").show(context, |_| {});
    egui::SidePanel::left("inventory_panel").show(context, |_| {});
    egui::SidePanel::right("inspector_panel").show(context, |_| {});
    egui::CentralPanel::default().show(context, |_| {});
}
}
"#,
    )?;
    std::fs::write(
        root.join("src").join("app").join("views").join("shell.rs"),
        r#"
fn render_shell(ui: &mut egui::Ui) {
ui.menu_button("Workspace", |_| {});
ui.menu_button("Node", |_| {});
ui.menu_button("View", |_| {});
}
"#,
    )?;
    std::fs::write(
        root.join("src").join("app").join("view.rs"),
        r#"
pub enum View { Summary }
impl View {
pub const ALL: [Self; 1] = [Self::Summary];
pub fn short_label(self) -> &'static str { "Sum" }
}
"#,
    )?;
    std::fs::write(
        root.join("src").join("app").join("paging.rs"),
        "pub fn page_count(total: usize, page_size: usize) -> usize { total.div_ceil(page_size).max(1) }\n",
    )?;
    if include_forbidden_markers {
        let scroll_marker = scroll_area_marker();
        std::fs::write(
            root.join("src")
                .join("app")
                .join("views")
                .join("legacy_webview.rs"),
            format!(
                "fn bad(ui: &mut egui::Ui) {{ {scroll_marker}vertical().show(ui, |_| {{}}); let _ = \"WebView\"; }}\n"
            ),
        )?;
    }
    Ok(())
}

fn scroll_area_marker() -> String {
    ["egui", "ScrollArea", ""].join("::")
}
