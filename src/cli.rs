use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};
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

// Define the possible sorting options
#[derive(ValueEnum, Clone, Debug, Default)]
pub enum SortBy {
    #[default] // Default to sorting by key (alphabetical)
    Key,
    Created,
    Modified,
}

#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("input_source")
        .required(true)
))]
pub struct ImportArgs {
    /// The path to the directory containing .md files.
    #[arg(long, group = "input_source")]
    pub dir: Option<String>,

    /// The path to a single .md file to import. (Requires --key)
    #[arg(long, group = "input_source", requires = "key")]
    pub file: Option<String>,

    /// The key to use for the single file import.
    #[arg(long)]
    pub key: Option<String>,

    /// Overwrite an existing note with the same key.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub overwrite: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    Markdown,
    Json,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// The path for the export directory or file.
    pub path: String,

    /// The output format.
    #[arg(long, value_enum, default_value_t = ExportFormat::Markdown)]
    pub format: ExportFormat,

    /// Export only notes with a specific tag.
    #[arg(long, short)]
    pub tag: Vec<String>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum TaskCommands {
    /// Add a new task linked to a note.
    Add {
        /// The key of the note this task is for.
        note_key: String,
        /// The description of the task.
        description: String,
    },
    /// List all open tasks.
    List,
    /// Mark a task as done.
    Done {
        /// The ID of the task to complete.
        task_id: u64,
    },
    Prio {
        /// The ID of the task to prioritize.
        task_id: u64,
    },
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
    echo \"This is a note from a pipe\" | medi new piped-note \n\n  \
    # With tags: Add tags to your note for better organization.\n  \
    medi new \"my-long-article\" --tag tag1 --tag tag2\n\n  \
    # With a title: Specify a title for your note.\n  \
    medi new \"my-long-article\" --title \"My Long Article\"\n")]
    New {
        /// The key (or title) for the new note.
        key: String,
        /// Provide the note content directly as an argument.
        #[arg(short, long)]
        message: Option<String>,
        #[arg(short = 'T', long)]
        tag: Vec<String>,
        #[arg(short, long)]
        title: Option<String>,
    },
    /// Edit an existing note with the specified key.
    #[command(after_help = "EXAMPLE:\n  \
    # Edit an existing note: Opens your default editor for long-form content.\n  \
    medi edit \"my-long-article\"\n\n  \
    # Add tags to a note: Adds one or more tags to the note.\n  \
    medi edit \"my-long-article\" --add-tag tag1 --add-tag tag2\n\n  \
    # Remove tags from a note: Removes one or more tags from the note.\n  \
    medi edit \"my-long-article\" --rm-tag tag1 --rm-tag tag2\n")]
    Edit {
        /// The key of the note to edit.
        key: String,
        /// Add one or more tags to the note.
        #[arg(long, short = 'a')]
        add_tag: Vec<String>,
        /// Remove one or more tags from the note.
        #[arg(long, short = 'r')]
        rm_tag: Vec<String>,
    },
    /// Get the content of a note with the specified key.
    #[command(after_help = "EXAMPLE:\n  \
    # Get a note: Displays the content of the note with the specified key.\n  \
    medi get \"my-long-article\"\n\n  \
    # Use this command to quickly view the content of a note without editing it.\n  \
    # Pipe to a Markdown renderer like mdcat \n  \
    medi get \"my-first-article\" | mdcat\n\n  \
    # You can also use this command to extract specific notes from a list.\n  \
    # For example, to get a note with a specific key:\n  \
    medi list | grep -o \"my-article\" | xargs medi get\n\n  \
    # Write the output to a file:\n  \
    medi get \"my-long-article\" > my-note.md\n\n  \
    # Use --json to output the note in JSON format:\n  \
    medi get \"my-long-article\" --json\n\n  \
    # Use --tag to retrieve all notes with a specific tag:\n  \
    medi get --tag my-tag\n")]
    Get {
        /// The key(s) of the note(s) to retrieve.
        #[arg(required_unless_present("tag"))]
        keys: Vec<String>,

        /// Retrieve all notes with a specific tag.
        #[arg(long, short, conflicts_with = "keys")]
        tag: Vec<String>,

        /// Output the full note data as JSON.
        #[arg(long, action = clap::ArgAction::SetTrue)]
        json: bool,
    },
    /// List all notes.
    #[command(after_help = "EXAMPLE:\n  \
    # List all notes: Displays a list of all notes in the database.\n  \
    medi list\n\n  \
    # Use this command to quickly see all your notes and their keys.\n  \
    # You can also pipe the output to other commands for further processing.\n  \
    medi list | grep -o \"my-article\" | xargs medi get\n\n  \
    # Use --sort-by to sort the notes by key, created date, or modified date\n \
    medi list --sort-by key")]
    List {
        /// The field to sort the notes by.
        #[arg(long, short, value_enum, default_value_t = SortBy::Key)]
        sort_by: SortBy,
    },
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
    /// Search for notes by content, title, or tags.
    #[command(after_help = "EXAMPLE:\n  \
    # Search for notes containing a specific term: Finds notes with 'meeting' in the content.\n  \
    medi search meeting")]
    Search {
        /// The search query string.
        query: String,
    },
    /// Reindex the search index.
    #[command(after_help = "EXAMPLE:\n  \
    # Reindex the search index: Rebuilds the search index from the existing notes.\n  \
    medi reindex\n\n  \
    # Use this command if you suspect the search index is out of sync with the notes.")]
    Reindex,
    /// Import notes from a directory or a single file.
    #[command(after_help = "EXAMPLE:\n  \
    # Import from a directory: Imports all .md files from the specified directory.\n  \
    medi import --dir /path/to/notes\n\n  \
    # Import a single file: Imports a single .md file with an mandatory key.\n  \
    medi import --file /path/to/note.md --key my-note\n\n  \
    # Use --overwrite to replace an existing note with the same key.\n  \
    medi import --file /path/to/note.md --key my-note --overwrite")]
    Import(ImportArgs),
    /// Export notes to a file.
    Export(ExportArgs),
    /// Manage tasks linked to notes.
    #[command(after_help = "EXAMPLE:\n  \
    # Add a new task linked to a note:\n  \
    medi task add my-note \"Finish writing the introduction\"\n\n  \
    # List all open tasks:\n  \
    medi task list\n\n  \
    # Mark a task as done:\n  \
    medi task done 1\n")]
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    /// Generates shell completion scripts.
    #[command(name = "generate-completion", hide = true)] // Hidden from help
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Update the medi application.
    #[command(name = "update", hide = true)] // Hidden from help
    /// Checks for a new version of medi and updates it if available.
    Update,
}
