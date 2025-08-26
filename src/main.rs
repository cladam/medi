use clap::Parser;
use medi::{colours, config, run, Cli};

/// Main entry point for medi
/// The application logic is contained in lib.rs, and this file is a thin wrapper responsible
/// only for parsing arguments and handling top-level errors.
fn main() {
    let config = match config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            colours::error(&format!("Failed to load configuration: {}", e));
            std::process::exit(1);
        }
    };
    let cli = Cli::parse();

    if let Err(e) = run(cli, config) {
        colours::error(&format!("Error: {}", e));
        std::process::exit(1);
    }
}
