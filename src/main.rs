#[cfg(not(test))]
use neo_nexus::{manager::ManagerAction, NeoNexusApp};

#[cfg(test)]
fn main() {}

#[cfg(not(test))]
fn main() {
    match neo_nexus::manager::action_from_args(std::env::args()) {
        Ok(ManagerAction::LaunchGui) => {
            if let Err(error) = run_native_app() {
                eprintln!("NeoNexus failed to start: {error}");
                std::process::exit(1);
            }
        }
        Ok(action) => {
            if let Some(output) = action.into_cli_output() {
                print!("{}", output.text_with_trailing_newline());
                if output.should_exit_process() {
                    std::process::exit(output.exit_code());
                }
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

#[cfg(not(test))]
fn run_native_app() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([1280.0, 760.0]),
        ..Default::default()
    };
    eframe::run_native(
        "NeoNexus",
        options,
        Box::new(|_creation_context| Ok(Box::new(NeoNexusApp::open_default()?))),
    )
}
