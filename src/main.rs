use medi::{run, Cli};
use clap::Parser;

/// Main entry point for medi
fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}