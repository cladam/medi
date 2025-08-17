mod cli;
mod db;
mod error;
pub mod colours;
mod note;

use std::{fs, io};
use std::io::Read;
use std::path::{Path, PathBuf};
use atty::Stream;
use chrono::Utc;
use clap::CommandFactory;
use dialoguer::Confirm;
pub use cli::{Cli, Commands};
use error::AppError;
use tempfile::Builder as TempBuilder;
use crate::cli::ExportFormat;
use crate::note::{JsonExport, Note};

// The main logic function, which takes the parsed CLI commands
pub fn run(cli: Cli) -> Result<(), AppError> {
    // Open the database
    let db = db::open()?;

    match cli.command {
        Commands::New { key, message, title, tag } => {
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
                db::save_note(&db, &new_note)?;
                colours::success(&format!("Successfully created note: '{}'", key));
            }
        }
        Commands::Edit { key, add_tag, rm_tag } => {
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
                db::save_note(&db, &existing_note)?;
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

                // This will now correctly overwrite the old note.
                db::save_note(&db, &existing_note)?;
                colours::success(&format!("Successfully updated note: '{}'", key));
            } else {
                colours::info("Note content unchanged.");
            }
        }
        Commands::Get { key, json } => {
            let note = db::get_note(&db, &key)?;
            if json {
                // Output the full Note struct as pretty JSON
                let json_output = serde_json::to_string_pretty(&note)?;
                println!("{}", json_output);
            } else {
                println!("{}", note.content);
            }
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
            let notes = db::get_all_notes(&db)?;
            let note_count = notes.len();

            if note_count == 0 {
                colours::warn("No notes to export.");
                return Ok(());
            }

            // Use a match statement to handle the different export formats
            match args.format {
                ExportFormat::Markdown => {
                    let export_path = Path::new(&args.path);
                    fs::create_dir_all(export_path)?;

                    // The loop variable is now a `Note` struct
                    for note in notes {
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
                        export_date : Utc::now(),
                        note_count,
                        notes
                    };

                    let json_string = serde_json::to_string_pretty(&export_data)?;
                    fs::write(&path, json_string)?;

                    colours::success(&format!(
                        "Successfully exported {} notes as JSON to '{}'",
                        note_count, path.display()
                    ));
                }
            }
        }
        Commands::Completion { shell } => {
            let mut cmd = cli::Cli::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, bin_name, &mut io::stdout());
        }
        Commands::Migrate => {
            colours::info("Starting migration of old notes to new format...");
            let mut migrated_count = 0;

            // Iterate over every raw entry in the database
            for result in db.iter() {
                let (key_bytes, val_bytes) = result?;

                // Try to parse the value as a Note. If it fails, it's an old raw string.
                if serde_json::from_slice::<Note>(&val_bytes).is_err() {
                    // It's an old note, let's migrate it.
                    let key = String::from_utf8(key_bytes.to_vec())?;
                    let content = String::from_utf8(val_bytes.to_vec())?;

                    let new_note = Note {
                        key: key.clone(),
                        title: key.clone(), // Default title to key
                        tags: vec![],
                        content,
                        created_at: Utc::now(),
                        modified_at: Utc::now(),
                    };

                    // Save the new, structured Note back to the database, overwriting the old one.
                    db::save_note(&db, &new_note)?;
                    migrated_count += 1;
                    println!("- Migrated '{}'", key);
                }
            }

            if migrated_count > 0 {
                colours::success(&format!("Migration complete. Migrated {} notes.", migrated_count));
            } else {
                colours::warn("No notes needed migration.");
            }
        }
    }
    Ok(())
}