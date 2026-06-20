use super::markers::RequiredMarker;

pub(in crate::native_ui) fn required_markers() -> [RequiredMarker; 10] {
    [
        RequiredMarker {
            path: "src/main.rs",
            alternate_paths: &[],
            marker: "eframe::run_native",
            message: "Application entry point must launch an eframe native window.",
        },
        RequiredMarker {
            path: "src/main.rs",
            alternate_paths: &[],
            marker: "eframe::NativeOptions",
            message: "Application entry point must configure native window options.",
        },
        RequiredMarker {
            path: "src/main.rs",
            alternate_paths: &[],
            marker: ".with_min_inner_size",
            message: "Application window must define a stable minimum desktop size.",
        },
        RequiredMarker {
            path: "src/app/frame.rs",
            alternate_paths: &["src/app.rs"],
            marker: "impl eframe::App",
            message: "Application state must implement the native eframe App lifecycle.",
        },
        RequiredMarker {
            path: "src/app/frame.rs",
            alternate_paths: &["src/app.rs"],
            marker: "egui::TopBottomPanel::top",
            message: "Application shell must keep a fixed top command/header panel.",
        },
        RequiredMarker {
            path: "src/app/frame.rs",
            alternate_paths: &["src/app.rs"],
            marker: "egui::TopBottomPanel::bottom",
            message: "Application shell must keep a fixed bottom status panel.",
        },
        RequiredMarker {
            path: "src/app/frame.rs",
            alternate_paths: &["src/app.rs"],
            marker: "egui::SidePanel::left",
            message: "Application shell must keep a fixed left inventory panel.",
        },
        RequiredMarker {
            path: "src/app/frame.rs",
            alternate_paths: &["src/app.rs"],
            marker: "egui::SidePanel::right",
            message: "Application shell must keep a fixed right inspector panel.",
        },
        RequiredMarker {
            path: "src/app/frame.rs",
            alternate_paths: &["src/app.rs"],
            marker: "egui::CentralPanel::default",
            message: "Application shell must keep one central fixed workspace panel.",
        },
        RequiredMarker {
            path: "src/app/view.rs",
            alternate_paths: &[],
            marker: "const ALL",
            message: "Application workspaces must use an explicit fixed native view set.",
        },
    ]
}
