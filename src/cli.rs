use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "medi",
    author = "Claes Adamsson @cladam",
    version,
    about = "CLI driven Markdown manager",
    long_about = None)]
#[command(propagate_version = true)]

pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new note with the specified key.
    #[command(after_help = "EXAMPLE:\n  \
    # Interactively (default): Opens your default editor for long-form content.\n  \
    medi new \"my-long-article\"\n\n  \
    # With a direct message: Perfect for quick, one-line notes. \n  \
    medi new quick-idea -m \"Remember to buy milk\"\n\n  \
    # From a pipe: Use the output of other commands as your note content.\n  \
    echo \"This is a note from a pipe\" | medi new piped-note")]
    New {
        /// The key (or title) for the new note.
        key: String,
        /// Provide the note content directly as an argument.
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Edit an existing note with the specified key.
    #[command(after_help = "EXAMPLE:\n  \
    # Edit an existing note: Opens your default editor for long-form content.\n  \
    medi edit \"my-long-article\"")]
    Edit { key: String },
    /// Get the content of a note with the specified key.
    #[command(after_help = "EXAMPLE:\n  \
    # Get a note: Displays the content of the note with the specified key.\n  \
    medi get \"my-long-article\"\n\n  \
    # Use this command to quickly view the content of a note without editing it.\n  \
    # Pipe to a Markdown renderer like mdcat \n  \
    medi get \"my-first-article\" | mdcat")]
    Get { key: String },
    /// List all notes.
    #[command(after_help = "EXAMPLE:\n  \
    # List all notes: Displays a list of all notes in the database.\n  \
    medi list\n\n  \
    # Use this command to quickly see all your notes and their keys.\n  \
    # You can also pipe the output to other commands for further processing.\n  \
    medi list | grep -o \"my-article\" | xargs medi get")]
    List,
    /// Delete a note with the specified key.
    #[command(after_help = "EXAMPLE:\n  \
    # Delete a note: Removes the note with the specified key.\n  \
    medi delete \"my-long-article\"\n\n  \
    # Use --force to skip confirmation.\n  \
    medi delete \"my-long-article\" --force\n\n  \
    # Note: Use this command with caution, as it will permanently delete the note.")]
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
    /// Generates shell completion scripts.
    #[command(name = "generate-completion", hide = true)] // Hidden from help
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
}