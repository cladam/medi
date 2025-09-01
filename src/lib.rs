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
use crate::task::{Task, TaskStatus};
use atty::Stream;
use chrono::Utc;
use clap::CommandFactory;
pub use cli::{Cli, Commands};
use colored::Colorize;
use config::Config;
use crossbeam_channel::unbounded;
use dialoguer::Confirm;
use error::AppError;
use regex::Regex;
#[cfg(unix)]
use skim::options::SkimOptionsBuilder;
#[cfg(unix)]
use skim::{Skim, SkimItem};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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

/// Helper function to calculate reading time
fn calculate_reading_time(word_count: usize) -> u64 {
    // Assuming an average reading speed of 225 words per minute
    let wpm = 225.0;
    (word_count as f64 / wpm).ceil() as u64
}

/// Helper function to count words in a string
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
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
            template,
        } => {
            // Check for key existence here
            if db::key_exists(&db, &key)? {
                return Err(AppError::KeyExists(key));
            }

            // Determine the final content based on the input method.
            let content = if let Some(message_content) = message {
                message_content
            } else if !atty::is(Stream::Stdin) {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            } else {
                // Open the editor.
                let initial_content = if let Some(template_name) = template {
                    let config_dir = dirs::config_dir().ok_or_else(|| {
                        AppError::ConfigError("Config directory not found".into())
                    })?;
                    let template_path = config_dir
                        .join("medi/templates")
                        .join(format!("{}.md", template_name));

                    // Read the template file, return empty string if it fails (e.g. not found).
                    fs::read_to_string(template_path).unwrap_or_default()
                } else {
                    // No template, so start with a blank editor.
                    String::new()
                };

                // Now, open the editor with the initial content.
                let tempfile = TempBuilder::new()
                    .prefix("medi-note-")
                    .suffix(".md")
                    .tempfile()?;
                let temppath = tempfile.path().to_path_buf();
                // Write the initial content (template or empty) to the temp file.
                fs::write(&temppath, &initial_content)?;
                // Open the pre-filled temp file in the editor.
                edit::edit_file(&temppath)?;
                // Read the final content back.
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
        Commands::Backlinks { key } => {
            let all_notes = db::get_all_notes(&db)?;

            // The pattern we're looking for is [[key]]
            let link_pattern = format!(r"\[\[{}\]\]", regex::escape(&key));
            let re = Regex::new(&link_pattern)?;

            let mut linking_notes = Vec::new();
            for note in all_notes {
                // Don't link a note to itself
                if note.key == key {
                    continue;
                }
                // If the note's content contains a link to our key, add it to the list.
                if re.is_match(&note.content) {
                    linking_notes.push(note.key);
                }
            }

            if linking_notes.is_empty() {
                colours::warn(&format!("No backlinks found for '{}'.", key));
            } else {
                colours::info(&format!(
                    "Found {} backlinks for '{}':",
                    linking_notes.len(),
                    key.bold()
                ));
                for linking_key in linking_notes {
                    println!("- {}", linking_key);
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

            // Get all notes from the primary database.
            let all_notes = db::get_all_notes(&db)?;
            let note_count = all_notes.len();

            // Get a writer and wipe the old index.
            let mut index_writer: tantivy::IndexWriter<tantivy::TantivyDocument> =
                search_index.writer(100_000_000)?; // 100MB heap
            index_writer.delete_all_documents()?;

            // Add all notes to the index.
            for note in all_notes {
                search::add_note_to_index(&note, &mut index_writer)?;
            }

            index_writer.commit()?;

            colours::success(&format!("Successfully reindexed {} notes.", note_count));
        }
        #[cfg(unix)]
        Commands::Find => {
            let notes = db::get_all_notes(&db)?;
            if notes.is_empty() {
                colours::warn("No notes to find.");
                return Ok(());
            }

            // Create a crossbeam channel.
            let (tx, rx) = unbounded();

            // Send each note key through the channel.
            for note in notes {
                let item: Arc<dyn SkimItem> = Arc::new(note.key);
                let _ = tx.send(item);
            }
            drop(tx);

            // Configure and run the fuzzy finder.
            let options = SkimOptionsBuilder::default()
                .height("30%".to_string())
                .prompt("Select a note to edit: ".to_string())
                .reverse(true)
                .border(Some("─".to_string()))
                .multi(false)
                .build()
                .unwrap();

            // `Skim::run_with` launches the interactive fuzzy finder.
            // We pass the receiver `rx` which `skim` will use to get the items.
            let selected_items = Skim::run_with(&options, Some(rx))
                .map(|out| out.selected_items)
                .unwrap_or_default();

            // Get the selected key and open it for editing.
            if let Some(item) = selected_items.first() {
                let selected_key = item.output().to_string();
                let mut existing_note = db::get_note(&db, &selected_key)?;

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
                    db::save_note_with_index(&db, &existing_note, &search_index)?;
                    colours::success(&format!("Successfully updated note: '{}'", selected_key));
                } else {
                    colours::info("Note content unchanged.");
                }
            } else {
                colours::info("No note selected.");
            }
        }
        #[cfg(not(unix))]
        Commands::Find => {
            return Err(AppError::Unsupported(
                "The 'find' command is not supported on this operating system.".to_string(),
            ));
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
            cli::TaskCommands::Add {
                note_key,
                description,
            } => {
                // First, make sure the note exists.
                db::get_note(&db, &note_key)?;

                let new_task = Task {
                    id: db::get_next_task_id(&db)?,
                    note_key,
                    description,
                    status: TaskStatus::Open,
                    created_at: Utc::now(),
                };
                db::save_task(&db, &new_task)?;
                colours::success(&format!("Added new task with ID: {}", new_task.id));
            }
            cli::TaskCommands::List => {
                let mut tasks = db::get_all_tasks(&db)?;
                let open_tasks: Vec<_> = tasks.clone().clone().into_iter().collect();

                if open_tasks.is_empty() {
                    colours::info("No open tasks.");
                } else {
                    // Sort tasks by status
                    tasks.sort_by_key(|t| match t.status {
                        TaskStatus::Prio => 0,
                        TaskStatus::Open => 1,
                        TaskStatus::Done => 2,
                    });
                    colours::info("Open tasks:");
                    for task in open_tasks {
                        // Format the status with colour
                        let status_str = match task.status {
                            TaskStatus::Open => "[Open] ".cyan(),
                            TaskStatus::Prio => "[Prio] ⭐ ".yellow().bold(),
                            TaskStatus::Done => "[Done] ".green(),
                        };
                        println!(
                            "- {}{}: {} (for note {})",
                            status_str,
                            task.id,
                            task.description,
                            task.note_key.cyan().bold()
                        );
                    }
                }
            }
            cli::TaskCommands::Done { task_id } => {
                let tasks = db::get_all_tasks(&db)?;
                if let Some(mut task) = tasks.into_iter().find(|t| t.id == task_id) {
                    task.status = TaskStatus::Done;
                    db::save_task(&db, &task)?;
                    colours::success(&format!("Completed task: {}", task_id));
                } else {
                    Err(AppError::TaskNotFound(task_id))?;
                }
            }
            cli::TaskCommands::Prio { task_id } => {
                let tasks = db::get_all_tasks(&db)?;
                if let Some(mut task) = tasks.into_iter().find(|t| t.id == task_id) {
                    task.status = TaskStatus::Prio;
                    db::save_task(&db, &task)?;
                    colours::success(&format!("Prioritised task: {}", task_id));
                } else {
                    Err(AppError::TaskNotFound(task_id))?;
                }
            }
            cli::TaskCommands::Delete { task_id } => {
                let tasks = db::get_all_tasks(&db)?;
                if tasks.iter().any(|t| t.id == task_id) {
                    db::delete_task(&db, task_id)?;
                    colours::success(&format!("Deleted task: {}", task_id));
                } else {
                    Err(AppError::TaskNotFound(task_id))?;
                }
            }
            cli::TaskCommands::Reset { force } => {
                let confirmed = if force {
                    true
                } else {
                    Confirm::new()
                        .with_prompt("Are you sure you want to reset all tasks?")
                        .default(false)
                        .interact()?
                };
                if confirmed {
                    db::delete_all_tasks(&db)?;
                    colours::success("All tasks have been reset.");
                } else {
                    colours::warn("Task reset cancelled.");
                }
            }
        },
        Commands::Status { key } => {
            if let Some(note_key) = key {
                // --- DETAILED NOTE STATS ---
                let note = db::get_note(&db, &note_key)?;
                let word_count = count_words(&note.content).into();
                let reading_time = calculate_reading_time(word_count);
                let tags_str = if note.tags.is_empty() {
                    "None".to_string()
                } else {
                    note.tags.join(", ")
                };

                println!("{}", note.title.bold().underline());
                println!("  Key: {}", note.key.cyan());
                println!("  Tags: {}", tags_str.cyan());
                println!("  Words: {}", word_count.to_string().cyan());
                println!(
                    "  Reading Time: ~{} minute(s)",
                    reading_time.to_string().cyan()
                );
                println!("  Created: {}", note.created_at.to_rfc2822());
                println!("  Modified: {}", note.modified_at.to_rfc2822());
            } else {
                // --- GLOBAL DATABASE OVERVIEW ---
                let notes = db::get_all_notes(&db)?;
                let tasks = db::get_all_tasks(&db)?;
                let open_tasks: Vec<_> = tasks
                    .iter()
                    .filter(|t| !matches!(t.status, TaskStatus::Done))
                    .collect();
                let prio_tasks_count = open_tasks
                    .iter()
                    .filter(|t| matches!(t.status, TaskStatus::Prio))
                    .count();

                println!("{}", "medi status".bold().underline());
                println!("  Notes: {}", notes.len().to_string().cyan());
                println!(
                    "  Tasks: {} open ({} priority)",
                    open_tasks.len().to_string().cyan(),
                    prio_tasks_count.to_string().yellow()
                );
            }
        }
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
