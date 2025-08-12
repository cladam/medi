use crate::error::AppError;
use sled::Db;
use std::{env, fs, str};
use std::path::PathBuf;

// Helper function to open the database
// It checks the environment variable `MEDI_DB_PATH` for the database path.
// If the variable is not set, it defaults to `~/.medi/medi_db`
// It ensures the parent directory exists before opening the database.
// If the database cannot be opened, it returns an AppError::Sled.
// If the home directory cannot be found, it returns an AppError::Io.
// If the database is opened successfully, it returns a sled::Db instance.
pub fn open() -> Result<Db, AppError> {
    let db_path = match env::var("MEDI_DB_PATH") {
        Ok(db_path) => PathBuf::from(db_path),
        Err(_) => {
            // Use the home directory.
            let home_dir = dirs::home_dir().ok_or_else(|| {
                AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                ))
            })?;
            home_dir.join(".medi").join("medi_db")
        }
    };

    // Ensure the parent directory exists.
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    sled::open(db_path).map_err(AppError::from)
}

/// This function just saves content. It's simple and has no editor logic.
pub fn insert_new_note(db: &Db, key: &str, content: &str) -> Result<(), AppError> {
    if db.contains_key(key)? {
        return Err(AppError::KeyExists(key.to_string()));
    }
    db.insert(key, content.as_bytes())?;
    db.flush()?;
    Ok(())
}

/// This function updates an existing note in the database by its key.
/// Corresponds to `medi update <key> <new_content>`
/// It checks if the key exists in the database.
/// If the key does not exist, it returns an AppError::KeyNotFound.
/// If the key exists, it updates the note content with the new content provided.
/// If there is an error during the process, it returns an AppError.
pub fn update_note(db: &Db, key: &str, new_content: &str) -> Result<(), AppError> {
    if !db.contains_key(key)? {
        return Err(AppError::KeyNotFound(key.to_string()));
    }

    db.insert(key, new_content.as_bytes())?;
    db.flush()?;
    Ok(())
}

// This function retrieves a note from the database by its key.
// Corresponds to `medi get <key>`
// It returns the note content as a String.
// If the key does not exist, it returns an AppError::KeyNotFound.
// If there is an error reading the database or converting the content to a String, it returns an AppError.
pub fn get_note(db: &Db, key: &str) -> Result<String, AppError> {
    let value_ivec = db.get(key)?
        .ok_or_else(|| AppError::KeyNotFound(key.to_string()))?;

    Ok(str::from_utf8(&value_ivec)?.to_string())
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

/// Imports a single note, handling the overwrite logic.
/// Returns `Ok(true)` if imported, `Ok(false)` if skipped.
pub fn import_note(db: &Db, key: &str, content: &str, overwrite: bool) -> Result<bool, AppError> {
    let key_exists = db.contains_key(key)?;

    if key_exists && !overwrite {
        // Key exists and we shouldn't overwrite, so we skip it.
        return Ok(false);
    }

    // Otherwise, insert/overwrite the note.
    db.insert(key, content.as_bytes())?;
    db.flush()?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module (db)
    use sled::Config;

    #[test]
    fn test_insert_new_note_success() {
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();
        let key = "new-key";
        let content = "hello world";

        let result = insert_new_note(&db, key, content);
        assert!(result.is_ok());

        let saved_value = db.get(key).unwrap().unwrap();
        assert_eq!(saved_value, content.as_bytes());
    }

    #[test]
    fn test_insert_new_note_key_exists() {
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();
        let key = "existing-key";
        db.insert(key, "old value").unwrap();

        let result = insert_new_note(&db, key, "new content");
        assert!(matches!(result, Err(AppError::KeyExists(_))));
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
    fn test_update_note() {
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();
        let key = "existing-note";
        db.insert(key, "old content").unwrap();

        let result = update_note(&db, key, "new content");
        assert!(result.is_ok());

        // Verify the content was updated
        let updated_value = db.get(key).unwrap().unwrap();
        assert_eq!(updated_value, "new content".as_bytes());
    }
}