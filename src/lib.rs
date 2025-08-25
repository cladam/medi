mod cli;
pub mod colours;
pub mod config;
mod db;
mod error;
mod note;
mod search;
mod task;

use crate::cli::{ExportFormat, SortBy};
use crate::note::{JsonExport, Note};
use atty::Stream;
use chrono::Utc;
use clap::CommandFactory;
pub use cli::{Cli, Commands};
use colored::Colorize;
use config::Config;
use dialoguer::Confirm;
use error::AppError;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use tempfile::Builder as TempBuilder;

pub fn initialise_search_index(config: &Config) -> Result<tantivy::Index, AppError> {
    let search_index_path = match env::var("MEDI_DB_PATH") {
        Ok(path_str) => PathBuf::from(path_str).join("search_index"),
        Err(_) => config
            .db_path
            .as_ref()
            .map(|db_path| db_path.join("search_index"))
            .unwrap_or_else(|| {
                dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("medi")
                    .join("search_index")
            }),
    };

    let index = search::open_index(&search_index_path)?;
    Ok(index)
}

/// Formats a slice of tags into a colored, space-separated string.
fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        "".to_string()
    } else {
        format!(
            " [{}]",
            tags.iter()
                .map(|t| format!("#{}", t).cyan().to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

// The main logic function, which takes the parsed CLI commands
pub fn run(cli: Cli, config: Config) -> Result<(), AppError> {
    // Open the database
    let db = db::open(config.clone())?; // Clone config for search index init
                                        // Initialise the search index
    let search_index =
        initialise_search_index(&config).map_err(|e| AppError::Search(e.to_string()))?;

    match cli.command {
        Commands::New {
            key,
            message,
            title,
            tag,
        } => {
            // Check for key existence here, in the application logic.
            if db::key_exists(&db, &key)? {
                return Err(AppError::KeyExists(key));
            }

            // Determine the content from one of three sources.
            let content = if let Some(message_content) = message {
                message_content
            } else if !atty::is(Stream::Stdin) {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            } else {
                let tempfile = TempBuilder::new()
                    .prefix("medi-note-")
                    .suffix(".md")
                    .tempfile()?;
                let temppath = tempfile.path().to_path_buf();
                edit::edit_file(&temppath)?;
                fs::read_to_string(&temppath)?
            };

            // Save the note if content is not empty.
            if content.trim().is_empty() {
                colours::warn("Note creation cancelled (empty content).");
            } else {
                // Create a new Note instance with all the metadata
                let new_note = Note {
                    key: key.clone(),
                    // Use the title flag, or default to the key
                    title: title.unwrap_or_else(|| key.clone()),
                    tags: tag,
                    content,
                    created_at: Utc::now(),
                    modified_at: Utc::now(),
                };
                // Save the entire Note object
                db::save_note_with_index(&db, &new_note, &search_index)?;
                colours::success(&format!("Successfully created note: '{}'", key));
            }
        }
        Commands::Edit {
            key,
            add_tag,
            rm_tag,
        } => {
            let mut existing_note = db::get_note(&db, &key)?;
            let mut modified = false;

            // Handle adding tags
            if !add_tag.is_empty() {
                for tag in add_tag {
                    if !existing_note.tags.contains(&tag) {
                        existing_note.tags.push(tag);
                        modified = true;
                    }
                }
            }

            // Handle removing tags
            if !rm_tag.is_empty() {
                let original_len = existing_note.tags.len();
                // Retain only the tags that are NOT in the rm_tag list.
                existing_note.tags.retain(|tag| !rm_tag.contains(tag));
                if existing_note.tags.len() != original_len {
                    modified = true;
                }
            }

            if modified {
                existing_note.modified_at = Utc::now();
                db::save_note_with_index(&db, &existing_note, &search_index)?;
                colours::success(&format!("Successfully updated tags for '{}'", key));
                return Ok(());
            }

            // If no tags were modified, proceed to edit the content.
            let tempfile = TempBuilder::new()
                .prefix("medi-note-")
                .suffix(".md")
                .tempfile()?;

            let temppath = tempfile.path().to_path_buf();
            fs::write(&temppath, &existing_note.content)?;
            edit::edit_file(&temppath)?;

            let updated_content = fs::read_to_string(&temppath)?;
            if updated_content.trim() != existing_note.content.trim() {
                existing_note.content = updated_content;
                existing_note.modified_at = Utc::now();

                // This will overwrite the old note.
                db::save_note_with_index(&db, &existing_note, &search_index)?;
                colours::success(&format!("Successfully updated note: '{}'", key));
            } else {
                colours::info("Note content unchanged.");
            }
        }
        Commands::Get { keys, tag, json } => {
            let notes_to_show = if !tag.is_empty() {
                // If tags are provided, retrieve all notes with those tags
                let all_notes = db::get_all_notes(&db)?;
                all_notes
                    .into_iter()
                    .filter(|note| note.tags.iter().any(|t| tag.contains(t)))
                    .collect::<Vec<_>>()
            } else {
                // If keys are provided, retrieve those specific notes
                let mut notes = Vec::new();
                for key in keys {
                    notes.push(db::get_note(&db, &key)?);
                }
                notes
            };

            if notes_to_show.is_empty() {
                colours::warn("No matching notes found.");
                return Ok(());
            }

            // Print the filtered notes
            for (i, note) in notes_to_show.iter().enumerate() {
                if i > 0 {
                    println!("---");
                } // Separator for multiple notes
                if json {
                    println!("{}", serde_json::to_string_pretty(note)?);
                } else {
                    println!("{}", note.content);
                }
            }
        }
        Commands::List { sort_by } => {
            let mut notes = db::get_all_notes(&db)?;
            if notes.is_empty() {
                colours::warn("No notes found.");
            }

            // Sorting logic
            match sort_by {
                SortBy::Key => notes.sort_by(|a, b| a.key.cmp(&b.key)),
                SortBy::Created => notes.sort_by(|a, b| b.created_at.cmp(&a.created_at)), // Newest first
                SortBy::Modified => notes.sort_by(|a, b| b.modified_at.cmp(&a.modified_at)), // Newest first
            }

            // Print rich output
            println!("{}:", "Notes".bold().underline());
            for note in notes {
                // Format the tags into a colored string like `[#tag1 #tag2]`
                let tags_str = format_tags(&note.tags);

                // Print the formatted line
                println!("- {}{}", note.key.green().bold(), tags_str);
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
                if db::delete_note_with_index(&db, &key, &search_index).is_ok() {
                    colours::success(&format!("Successfully deleted note: '{}'", key));
                } else {
                    colours::error(&format!(
                        "Failed to delete note: '{}'. It may not exist.",
                        key
                    ));
                }
            } else {
                colours::warn("Deletion cancelled.");
            }
        }
        Commands::Search { query } => {
            let found_keys = search::search_notes(&search_index, &query)?;

            if found_keys.is_empty() {
                colours::warn("No matching notes found.");
                return Ok(());
            }

            println!("{}:", "Search Results".bold().underline());
            for key in found_keys {
                match db::get_note(&db, &key) {
                    Ok(note) => {
                        let tags_str = format_tags(&note.tags);
                        println!("- {}{}", note.key.green().bold(), tags_str);
                    }
                    Err(_) => {
                        colours::error(&format!(
                            "Found key '{}' in index, but failed to retrieve from database.",
                            key
                        ));
                    }
                }
            }
        }
        Commands::Reindex => {
            colours::info("Starting reindex of all notes...");

            // 1. Get all notes from the primary database.
            let all_notes = db::get_all_notes(&db)?;
            let note_count = all_notes.len();

            // 2. Get a writer and wipe the old index.
            let mut index_writer: tantivy::IndexWriter<tantivy::TantivyDocument> =
                search_index.writer(100_000_000)?; // 100MB heap
            index_writer.delete_all_documents()?;

            // 3. Add all notes to the index.
            for note in all_notes {
                // Use the dedicated function in the search module
                search::add_note_to_index(&note, &mut index_writer)?;
            }

            // 4. Commit the changes.
            index_writer.commit()?;

            colours::success(&format!("Successfully reindexed {} notes.", note_count));
        }
        Commands::Import(args) => {
            // This is a helper closure to handle the logic for a single file.
            let handle_import = |key: &str, content: &str| -> Result<(), AppError> {
                let note_exists = db::key_exists(&db, key)?;

                if note_exists && !args.overwrite {
                    colours::warn(&format!("Skipped '{}' (already exists)", key));
                    return Ok(());
                }

                // Create a new Note struct from the imported file content.
                let new_note = Note {
                    key: key.to_string(),
                    title: key.to_string(), // Default title to the key
                    tags: vec![],           // Default to no tags
                    content: content.to_string(),
                    created_at: Utc::now(),
                    modified_at: Utc::now(),
                };

                // Save the complete Note object.
                db::save_note(&db, &new_note)?;
                colours::success(&format!("Imported '{}'", key));
                Ok(())
            };

            if let (Some(file_path), Some(key)) = (args.file, args.key) {
                // Single file import
                let content = fs::read_to_string(&file_path)?;
                handle_import(&key, &content)?;
            } else if let Some(dir_path_str) = args.dir {
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
                            if let Err(e) = handle_import(key, &content) {
                                colours::error(&format!("Failed to import '{}': {}", key, e));
                            }
                        }
                    }
                }
            }
        }
        Commands::Export(args) => {
            let all_notes = db::get_all_notes(&db)?;

            // Filter notes by tag if the --tag flag was provided
            let notes_to_export = if !args.tag.is_empty() {
                all_notes
                    .into_iter()
                    .filter(|note| args.tag.iter().all(|t| note.tags.contains(t)))
                    .collect()
            } else {
                all_notes // Otherwise, export all notes
            };

            let note_count = notes_to_export.len();
            if note_count == 0 {
                colours::warn("No matching notes to export.");
                return Ok(());
            }

            // Use a match statement to handle the different export formats
            match args.format {
                ExportFormat::Markdown => {
                    let export_path = Path::new(&args.path);
                    fs::create_dir_all(export_path)?;

                    // The loop variable is now a `Note` struct
                    for note in notes_to_export {
                        // Use the note's key as the filename
                        let file_path = export_path.join(format!("{}.md", note.key));
                        // Write the note's .content, not the whole note object
                        fs::write(file_path, &note.content)?;
                    }
                    colours::success(&format!(
                        "Successfully exported {} notes as Markdown to '{}'",
                        note_count, args.path
                    ));
                }
                ExportFormat::Json => {
                    let mut path = PathBuf::from(&args.path);

                    if path.extension().and_then(|s| s.to_str()) != Some("json") {
                        path.set_extension("json");
                    }

                    let export_data = JsonExport {
                        export_date: Utc::now(),
                        note_count,
                        notes: notes_to_export,
                    };

                    let json_string = serde_json::to_string_pretty(&export_data)?;
                    fs::write(&path, json_string)?;

                    colours::success(&format!(
                        "Successfully exported {} notes as JSON to '{}'",
                        note_count,
                        path.display()
                    ));
                }
            }
        }
        Commands::Task { command } => match command {
            // Handle each subcommand from the `TaskCommands` enum.
            // For now, they all point to the placeholder message.
            cli::TaskCommands::Add { .. }
            | cli::TaskCommands::List { .. }
            | cli::TaskCommands::Done { .. }
            | cli::TaskCommands::Prio { .. } => {
                colours::warn("Task commands are not yet implemented.")
            }
        },
        Commands::Completion { shell } => {
            let mut cmd = cli::Cli::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, bin_name, &mut io::stdout());
        }
        Commands::Update => {
            println!("{}", "--- Checking for updates ---".blue());
            let status = self_update::backends::github::Update::configure()
                .repo_owner("cladam")
                .repo_name("medi")
                .bin_name("medi")
                .show_download_progress(true)
                .current_version(self_update::cargo_crate_version!())
                .build()?
                .update()?;

            println!("Update status: `{}`!", status.version());
            if status.updated() {
                println!("{}", "Successfully updated medi!".green());
            } else {
                println!("{}", "medi is already up to date.".green());
            }
        }
    }
    Ok(())
}
