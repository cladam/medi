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
    New {
        /// The key (or title) for the new note.
        key: String,
        /// Provide the note content directly as an argument.
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Edit an existing note with the specified key.
    Edit { key: String },
    /// Get the content of a note with the specified key.
    Get { key: String },
    /// List all notes.
    List,
    /// Delete a note with the specified key.
    Delete {
        /// The key of the note to delete.
        key: String,
        /// Skip the confirmation prompt.
        #[arg(long, short, action = clap::ArgAction::SetTrue)]
        force: bool,
    },
    /// Import notes from a file.
    Import { file: String },
    /// Export notes to a file.
    Export { file: String },
}