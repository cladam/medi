use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "medi",
    author = "Claes Adamsson @cladam",
    version,
    about = "A CLI tool for Trunk-Based Development (TBD) workflows",
    long_about = None)]
#[command(propagate_version = true)]

pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new note with the specified key.
    New { key: String },
    /// Edit an existing note with the specified key.
    Edit { key: String },
    /// Get the content of a note with the specified key.
    Get { key: String },
    /// List all notes.
    List,
    /// Delete a note with the specified key.
    Delete { key: String },
    /// Import notes from a file.
    Import { file: String },
    /// Export notes to a file.
    Export { file: String },
}