use neo_nexus::{cli::CliAction, NeoNexusApp};

fn main() {
    match neo_nexus::cli::action_from_args(std::env::args()) {
        Ok(CliAction::RunGui) => {
            if let Err(error) = run_native_app() {
                eprintln!("NeoNexus failed to start: {error}");
                std::process::exit(1);
            }
        }
        Ok(CliAction::Print(text)) => {
            print!("{text}");
            if !text.ends_with('\n') {
                println!();
            }
        }
        Ok(CliAction::PrintWithExitCode { text, exit_code }) => {
            print!("{text}");
            if !text.ends_with('\n') {
                println!();
            }
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

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
