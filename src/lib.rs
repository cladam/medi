mod cli;
mod db;
mod error;
pub mod colours;

use std::{fs, io};
use std::io::Read;
use std::path::Path;
use atty::Stream;
use clap::CommandFactory;
use dialoguer::Confirm;
pub use cli::{Cli, Commands};
use error::AppError;

// The main logic function, which takes the parsed CLI commands
pub fn run(cli: Cli) -> Result<(), AppError> {
    // Open the database
    let db = db::open()?;

    match cli.command {
        Commands::New { key, message } => {
            // Determine the content from one of three sources.
            let content = if let Some(message_content) = message {
                message_content
            } else if !atty::is(Stream::Stdin) {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            } else {
                // Fallback: call the editor directly from the application logic layer.
                edit::edit("")?
            };

            // Save the note if content is not empty.
            if content.trim().is_empty() {
                colours::warn("Note creation cancelled (empty content).");
            } else {
                // Call the simple insert function.
                db::insert_new_note(&db, &key, &content)?;
                colours::success(&format!("Successfully created note: '{}'", key));
            }
        }
        Commands::Edit { key } => {
            let existing_content = db::get_note(&db, &key)?;
            let updated_content = edit::edit(existing_content)?;
            if updated_content.trim().is_empty() {
                colours::warn("Note update cancelled (empty content).");
                return Ok(());
            }
            db::update_note(&db, &key, &updated_content)?;
            colours::success(&format!("Successfully updated note: '{}'", key));
        }
        Commands::Get { key } => {
            let content = db::get_note(&db, &key)?;
            println!("{}", content);
        }
        Commands::List => {
            let notes = db::list_notes(&db)?;
            if notes.is_empty() {
                colours::warn("No notes found.");
            } else {
                colours::info("Notes:");
                for note in notes {
                    println!("- {}", note);
                }
            }
        }
        Commands::Delete { key, force } => {
            let confirmed = if force {
                true
            } else {
                Confirm::new()
                    .with_prompt(format!("Are you sure you want to delete '{}'?", key))
                    .default(false)
                    .interact()?
            };

            if confirmed {
                if db::delete_note(&db, &key).is_ok() {
                    colours::success(&format!("Successfully deleted note: '{}'", key));
                } else {
                    colours::error(&format!("Failed to delete note: '{}'. It may not exist.", key));
                }
            } else {
                colours::warn("Deletion cancelled.");
            }
        }
        Commands::Import(args) => {
            if let (Some(file_path), Some(key)) = (args.file, args.key) {
                // Single file import
                let content = fs::read_to_string(&file_path)?;
                match db::import_note(&db, &key, &content, args.overwrite) {
                    Ok(true) => colours::success(&format!("Imported '{}' from '{}'", key, file_path)),
                    Ok(false) => colours::warn(&format!("Skipped '{}' (already exists)", key)),
                    Err(e) => colours::error(&format!("Failed to import '{}': {}", key, e)),
                }
            } else if let Some(dir_path_str) = args.directory {
                // Directory import
                let dir_path = Path::new(&dir_path_str);
                if !dir_path.is_dir() {
                    return Err(AppError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Directory not found: {}", dir_path_str),
                    )));
                }

                // Read the directory contents
                for entry in fs::read_dir(dir_path)? {
                    let entry = entry?;
                    let file_path = entry.path();

                    // Process only if it's a file with a .md extension
                    if file_path.is_file() && file_path.extension() == Some("md".as_ref()) {
                        // Use the filename (without extension) as the key
                        if let Some(key) = file_path.file_stem().and_then(|s| s.to_str()) {
                            let content = fs::read_to_string(&file_path)?;

                            // Call the database function to handle the insert
                            match db::import_note(&db, key, &content, args.overwrite) {
                                Ok(true) => colours::success(&format!("Imported '{}'", key)),
                                Ok(false) => colours::warn(&format!("Skipped '{}' (already exists)", key)),
                                Err(e) => colours::error(&format!("Failed to import '{}': {}", key, e)),
                            }
                        }
                    }
                }
            }
        }
        Commands::Export { .. } => {
            colours::info(&"Exporting notes is not implemented yet.".to_string());
        }
        Commands::Completion { shell } => {
            let mut cmd = cli::Cli::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, bin_name, &mut io::stdout());
        }
    }
    Ok(())
}