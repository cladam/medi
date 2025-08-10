use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    New { key: String },
    Edit { key: String },
    Get { key: String },
    List,
    Delete { key: String },
    Import { file: String },
    Export { file: String },
}