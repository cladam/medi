use medi::{run, config, Cli, colours};
use clap::Parser;

/// Main entry point for medi
/// The application logic is contained in lib.rs, and this file is a thin wrapper responsible
/// only for parsing arguments and handling top-level errors.
fn main() {
    // Load config at the very beginning.
    let config = config::load().expect("Could not load configuration");
    let cli = Cli::parse();

    if let Err(e) = run(cli, config) {
        colours::error(&format!("Error: {}", e));
        std::process::exit(1);
    }
}