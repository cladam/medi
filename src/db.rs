use crate::config::Config;
use crate::error::AppError;
use crate::note::Note;
use crate::search;
use crate::task::Task;
use serde_json;
use sled::Db;
use std::path::PathBuf;
use std::{env, fs, str};
use tantivy::{Index, IndexWriter, TantivyDocument};

// Helper function to open the database
// It checks the environment variable `MEDI_DB_PATH` for the database path.
// If the variable is not set, it defaults to `~/.medi/medi_db`
// It ensures the parent directory exists before opening the database.
// If the database cannot be opened, it returns an AppError::Sled.
// If the home directory cannot be found, it returns an AppError::Io.
// If the database is opened successfully, it returns a sled::Db instance.
pub fn open(config: Config) -> Result<Db, AppError> {
    let db_path = match env::var("MEDI_DB_PATH") {
        Ok(path_str) => PathBuf::from(path_str),
        Err(_) => config.db_path.clone().unwrap_or_else(|| {
            // Default path logic
            let mut path = dirs::home_dir().expect("Could not find home directory.");
            path.push(".medi/medi_db");
            path
        }),
    };

    // Ensure the parent directory exists.
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    sled::open(db_path).map_err(AppError::from)
}

/// Checks if a key exists in the database.
pub fn key_exists(db: &Db, key: &str) -> Result<bool, AppError> {
    db.contains_key(key).map_err(AppError::from)
}

/// Saves a Note object to the database by serializing it to JSON.
pub fn save_note(db: &Db, note: &Note) -> Result<(), AppError> {
    let json_bytes = serde_json::to_vec(note)?;

    db.insert(&note.key, json_bytes)?;
    db.flush()?;
    Ok(())
}

/// Saves a Note to the database and updates the search index.
pub fn save_note_with_index(db: &Db, note: &Note, index: &Index) -> Result<(), AppError> {
    // Save to the primary database first
    save_note(db, note)?;

    // Update the search index
    let mut index_writer: tantivy::IndexWriter<tantivy::TantivyDocument> =
        index.writer(50_000_000)?;

    // For updates, first delete the old document using the search module function.
    search::delete_note_from_index(&note.key, &mut index_writer)?;

    // Add the new/updated document using the search module function.
    search::add_note_to_index(note, &mut index_writer)?;

    // Commit changes to the index
    index_writer.commit()?;
    Ok(())
}

/// Deletes a note from the database and the search index.
pub fn delete_note_with_index(db: &Db, key: &str, index: &Index) -> Result<(), AppError> {
    // Delete from the primary database first
    match delete_note(db, key) {
        Ok(()) => {
            // Remove from the search index only if the note existed and was deleted
            let mut index_writer: IndexWriter<TantivyDocument> = index.writer(50_000_000)?;
            // Use the dedicated function from the search module
            search::delete_note_from_index(key, &mut index_writer)?;
            index_writer.commit()?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Retrieves a Note object from the database by deserializing it from JSON.
/// Corresponds to `medi get <key>`
/// It checks if the key exists in the database.
/// If the key does not exist, it returns an AppError::KeyNotFound.
/// If the key exists, it deserializes the note content from JSON and returns it.
/// If there is an error during the process, it returns an AppError.
pub fn get_note(db: &Db, key: &str) -> Result<Note, AppError> {
    let value_ivec = db
        .get(key)?
        .ok_or_else(|| AppError::KeyNotFound(key.to_string()))?;

    let note: Note = serde_json::from_slice(&value_ivec).map_err(AppError::from)?;
    Ok(note)
}

// This function deletes a note from the database by its key.
// Corresponds to `medi delete <key>`
// It checks if the key exists in the database.
// If the key does not exist, it returns an AppError::KeyNotFound.
// If the key exists, it removes the note from the database and flushes the changes.
// If there is an error during the process, it returns an AppError.
pub fn delete_note(db: &Db, key: &str) -> Result<(), AppError> {
    if !db.contains_key(key)? {
        return Err(AppError::KeyNotFound(key.to_string()));
    }
    db.remove(key)?;
    db.flush()?;
    Ok(())
}

/// Returns all notes as a vector of `Note` structs.
pub fn get_all_notes(db: &Db) -> Result<Vec<Note>, AppError> {
    db.iter()
        .values() // We only need the values, which are the serialized Note JSON
        .map(|result| {
            let value_bytes = result?;
            // Deserialize the JSON bytes into a Note struct
            let note: Note = serde_json::from_slice(&value_bytes)?;
            Ok(note)
        })
        .collect() // This will now correctly return a Result<Vec<Note>, AppError>
}

// -------------------- Tasks --------------------

/// Saves a task to the database.
pub fn save_task(db: &Db, task: &Task) -> Result<(), AppError> {
    let key = format!("tasks/{}", task.id);
    let json_bytes = serde_json::to_vec(task)?;
    db.insert(key, json_bytes)?;
    db.flush()?;
    Ok(())
}

/// Retrieves all tasks from the database.
pub fn get_all_tasks(db: &Db) -> Result<Vec<Task>, AppError> {
    db.scan_prefix("tasks/")
        .values()
        .map(|result| {
            let value_bytes = result?;
            let task: Task = serde_json::from_slice(&value_bytes)?;
            Ok(task)
        })
        .collect()
}

/// A simple way to get the next available ID for a new task.
/// This uses sled's built-in ID generation feature.
/// It is amazing but gives u64 IDs, which is overkill for our needs, no one wants ID 2000001 for a task.
/*pub fn get_next_task_id_sled(db: &Db) -> Result<u64, AppError> {
    // This is a simple counter stored at a known key.
    let id = db.generate_id()?;
    Ok(id)
}*/

/// Get the next available ID using a homegrown method.
/// Got help by Gemini for this one.
pub fn get_next_task_id(db: &Db) -> Result<u64, AppError> {
    const TASK_COUNTER_KEY: &[u8] = b"__counter__/tasks";

    // `update_and_fetch` is an atomic operation, which makes it safe
    // to use even if multiple programs were running at once.
    let new_id_bytes = db.update_and_fetch(TASK_COUNTER_KEY, |old_value| {
        // If there's an old value, parse it. Otherwise, start at 0.
        let old_id = match old_value {
            Some(bytes) => {
                // Try to parse the bytes as an u64
                let mut buf = [0u8; 8];
                buf.copy_from_slice(bytes);
                u64::from_le_bytes(buf)
            }
            None => 0,
        };

        // Increment the ID and save it back as bytes.
        let new_id = old_id + 1;
        Some(new_id.to_le_bytes().to_vec())
    })?;

    // Handle the Option and extract bytes from IVec
    let new_id = match new_id_bytes {
        Some(ivec) => {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&ivec);
            u64::from_le_bytes(buf)
        }
        None => {
            return Err(AppError::Database(
                "Failed to update task counter".to_string(),
            ))
        }
    };

    Ok(new_id)
}

/// Resets the task ID counter to 0.
/// This is mainly useful for testing purposes.
/// In a real-world scenario, resetting the counter could lead to ID collisions.
pub fn reset_task_counter(db: &Db) -> Result<(), AppError> {
    const TASK_COUNTER_KEY: &[u8] = b"__counter__/tasks";
    db.insert(TASK_COUNTER_KEY, &0u64.to_le_bytes())?;
    db.flush()?;
    Ok(())
}

/// Deletes all tasks from the database.
/// Only use this in testing or if you really want to clear all tasks.
pub fn delete_all_tasks(db: &Db) -> Result<usize, AppError> {
    let mut count = 0;
    // Find all keys with the "tasks/" prefix.
    let keys_to_delete: Vec<_> = db
        .scan_prefix("tasks/")
        .keys()
        .collect::<Result<Vec<_>, _>>()?;

    // Create a batch to delete them all at once.
    let mut batch = sled::Batch::default();
    for key in keys_to_delete {
        batch.remove(key.clone());
        count += 1;
    }

    // Apply the batch deletion.
    db.apply_batch(batch)?;
    db.flush()?;
    Ok(count)
}

// -------------------- Tests --------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::Note;
    use chrono::Utc;
    use sled::Config;

    #[test]
    fn test_save_and_get_note_success() {
        // Setup
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();
        let key = "test-key".to_string();

        // Create a Note object to save
        let new_note = Note {
            key: key.clone(),
            title: "Test Title".to_string(),
            tags: vec!["testing".to_string()],
            content: "Mock note content".to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        // Execute save_note
        let save_result = save_note(&db, &new_note);
        assert!(save_result.is_ok());

        // Verify by getting the note back
        let retrieved_note = get_note(&db, &key).unwrap();
        assert_eq!(retrieved_note.content, "Mock note content");
        assert_eq!(retrieved_note.tags, vec!["testing"]);
    }

    #[test]
    fn test_get_all_notes_success() {
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();

        // Create and save two notes.
        let note1 = Note {
            key: "note-a".to_string(),
            title: "Note A".to_string(),
            content: "content a".to_string(),
            tags: vec![],
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };
        let note2 = Note {
            key: "note-b".to_string(),
            title: "Note B".to_string(),
            content: "content b".to_string(),
            tags: vec![],
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };
        save_note(&db, &note1).unwrap();
        save_note(&db, &note2).unwrap();

        let all_notes = get_all_notes(&db).unwrap();

        assert_eq!(all_notes.len(), 2);
        // Check if we can find one of the notes by its key.
        assert!(all_notes.iter().any(|n| n.key == "note-a"));
    }

    #[test]
    fn test_get_all_notes_empty_db() {
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();

        let all_notes = get_all_notes(&db).unwrap();
        assert!(all_notes.is_empty());
    }

    #[test]
    fn test_delete_note_success() {
        let config = Config::new().temporary(true);
        let db = config.open().expect("Failed to open temporary db");
        let key = "test-delete-key";
        db.insert(key, "content").unwrap();
        let result = delete_note(&db, key);
        assert!(result.is_ok());
        assert!(!db.contains_key(key).unwrap());
    }

    #[test]
    fn test_update_note_success() {
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();
        let key = "my-key".to_string();

        let original_note = Note {
            key: key.clone(),
            title: "Original Title".to_string(),
            content: "original content".to_string(),
            tags: vec![],
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };
        save_note(&db, &original_note).unwrap();

        let updated_note = Note {
            key: key.clone(),
            title: "Updated Title".to_string(),
            content: "updated content".to_string(),
            tags: vec!["updated".to_string()],
            created_at: original_note.created_at, // creation time should not change
            modified_at: Utc::now(),
        };

        let result = save_note(&db, &updated_note);
        assert!(result.is_ok());

        let retrieved_note = get_note(&db, &key).unwrap();
        assert_eq!(retrieved_note.content, "updated content");
        assert_eq!(retrieved_note.title, "Updated Title");
        assert_eq!(retrieved_note.tags, vec!["updated"]);
    }
}
