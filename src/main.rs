use neo_nexus::manager::ManagerAction;

fn main() {
    match neo_nexus::manager::action_from_args(std::env::args()) {
        Ok(ManagerAction::ServeWeb(launch)) => {
            if let Err(error) = neo_nexus::web::run_web_server(launch) {
                eprintln!("NeoNexus web server failed: {error:#}");
                std::process::exit(1);
            }
        }
        Ok(action) => {
            if let Some(output) = action.into_cli_output() {
                use std::io::Write;
                let text = output.text_with_trailing_newline();
                let mut stdout = std::io::stdout();
                if stdout
                    .write_all(text.as_bytes())
                    .and_then(|()| stdout.flush())
                    .is_err()
                {
                    // The reader closed the pipe (for example `neo-nexus --help |
                    // head`); exit quietly like a standard CLI tool rather than
                    // panicking on the broken pipe.
                    std::process::exit(0);
                }
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
