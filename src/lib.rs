mod cli;
mod db;
mod error;
pub mod colours;

use std::io;
use std::io::Read;
use atty::Stream;
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
        // Commands::New { key , message} => {
        //     // Call the appropriate function from the `db` module
        //     db::create_note(&db, &key, |s| edit::edit(s))?;
        //     colours::success(&format!("Successfully created note: '{}'", key));
        // }
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
                true // If --force is used, we're automatically confirmed.
            } else {
                // Use dialoguer to ask the user for confirmation.
                Confirm::new()
                    .with_prompt(format!("Are you sure you want to delete '{}'?", key))
                    .default(false) // Default to "no" if the user just hits Enter.
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
        // The import/export commands can be implemented later
        Commands::Import { .. } => {
            colours::info(&"Importing notes is not implemented yet.".to_string());
        }
        Commands::Export { .. } => {
            colours::info(&"Exporting notes is not implemented yet.".to_string());
        }
    }
    Ok(())
}