use crate::error::AppError;
use sled::Db;
use std::{env, fs, str};
use std::path::PathBuf;
use serde_json;
use crate::config::Config;
use crate::note::Note;

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

/// Retrieves a Note object from the database by deserializing it from JSON.
/// Corresponds to `medi get <key>`
/// It checks if the key exists in the database.
/// If the key does not exist, it returns an AppError::KeyNotFound.
/// If the key exists, it deserializes the note content from JSON and returns it.
/// If there is an error during the process, it returns an AppError.
pub fn get_note(db: &Db, key: &str) -> Result<Note, AppError> {
    let value_ivec = db.get(key)?
        .ok_or_else(|| AppError::KeyNotFound(key.to_string()))?;

    let note: Note = serde_json::from_slice(&value_ivec).map_err(AppError::from)?;
    Ok(note)
}

// This function lists all notes in the database.
// Corresponds to `medi list`
// It returns a vector of note keys (strings).
// If the database is empty, it returns an empty vector.
// If there is an error reading the database, it returns an AppError.
pub fn list_notes(db: &Db) -> Result<Vec<String>, AppError> {
    db.iter()
        .keys()
        .map(|result| {
            let key_bytes = result?;
            let key_str = str::from_utf8(&key_bytes)?;
            Ok(key_str.to_string())
        })
        .collect()
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
    fn test_list_notes() {
        let config = Config::new().temporary(true);
        let db = config.open().expect("Failed to open temporary db");

        db.insert("zeta-key", "").unwrap();
        db.insert("alpha-key", "").unwrap();
        db.insert("gamma-key", "").unwrap();

        let keys = list_notes(&db).unwrap();

        assert_eq!(keys.len(), 3);
        assert_eq!(keys, vec!["alpha-key", "gamma-key", "zeta-key"]);
    }

    #[test]
    fn test_list_notes_empty_db() {
        let config = Config::new().temporary(true);
        let db = config.open().expect("Failed to open temporary db");

        let keys = list_notes(&db).unwrap();
        assert!(keys.is_empty());
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