use super::markers::ForbiddenMarker;

pub(in crate::native_ui_audit) fn forbidden_markers() -> Vec<ForbiddenMarker> {
    vec![
        ForbiddenMarker {
            marker: ["egui", "ScrollArea", ""].join("::"),
            message: "Native workspaces must remain fixed-panel application layouts.",
        },
        ForbiddenMarker {
            marker: ["ScrollArea", ""].join("::"),
            message: "Native workspaces must remain fixed-panel application layouts.",
        },
        ForbiddenMarker {
            marker: ["egui_extras", "TableBuilder", ""].join("::"),
            message: "Virtual scrolling tables are not part of the fixed application shell.",
        },
        ForbiddenMarker {
            marker: ["TableBuilder", ""].join("::"),
            message: "Virtual scrolling tables are not part of the fixed application shell.",
        },
        ForbiddenMarker {
            marker: call_marker("show_rows"),
            message: "Virtual row rendering indicates document-style scrolling UI.",
        },
        ForbiddenMarker {
            marker: call_marker("vertical_scroll"),
            message: "Viewport scrolling is outside the fixed-panel application model.",
        },
        ForbiddenMarker {
            marker: call_marker("horizontal_scroll"),
            message: "Viewport scrolling is outside the fixed-panel application model.",
        },
        ForbiddenMarker {
            marker: "WebView".to_string(),
            message: "WebView shells are outside the pure Rust native application boundary.",
        },
        ForbiddenMarker {
            marker: "webview".to_string(),
            message: "WebView shells are outside the pure Rust native application boundary.",
        },
        ForbiddenMarker {
            marker: ["tauri", ""].join("::"),
            message: "Tauri shells are outside the pure Rust native application boundary.",
        },
        ForbiddenMarker {
            marker: ["wry", ""].join("::"),
            message: "Wry WebView shells are outside the pure Rust native application boundary.",
        },
    ]
}

fn call_marker(name: &str) -> String {
    let mut marker = name.to_string();
    marker.push('(');
    marker
}
