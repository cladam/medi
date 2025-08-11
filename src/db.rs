use crate::error::AppError;
use sled::Db;
use std::{env, fs, str};
use std::path::PathBuf;
use anyhow::anyhow;

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

// This function creates a new note in the database.
// It takes a key and a closure that provides the editor function.
// Corresponds to `medi new <key>`
// The closure is called with an empty string, and it should return the note content.
// If the key already exists, it returns an AppError::KeyExists.
// If the note content is empty, it prints a message and returns Ok(()) without saving.
// If the note is successfully created, it saves the content to the database and flushes it.
// If there is an error during the process, it returns an AppError.
pub fn create_note<F>(db: &Db, key: &str, editor_fn: F) -> Result<(), AppError>
where
    F: for<'a> FnOnce(&'a str) -> Result<String, std::io::Error>,
{
    if db.contains_key(key)? {
        return Err(AppError::KeyExists(key.to_string()));
    }

    // Call the provided editor function instead of `edit::edit` directly.
    let value = editor_fn("")?;

    if value.trim().is_empty() {
        println!("Note creation cancelled (empty content).");
        return Ok(());
    }

    db.insert(key, value.as_bytes())?;
    db.flush()?;
    Ok(())
}

// Corresponds to `medi edit <key>`
pub fn edit_note(db: &Db, key: &str) -> Result<(), AppError> {
    // TODO: Get the existing note content from the db.
    // If not found, return AppError::KeyNotFound.
    // Use edit::edit(existing_content) to open the editor.
    // Save the new content back to the database.
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

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module (db)
    use sled::Config;

    #[test]
    fn test_create_note_success() {
        // 1. Use a temporary database for this test.
        let config = Config::new().temporary(true);
        let db = config.open().expect("Failed to open temporary db");
        let key = "test-key";

        fn mock_editor(_: &str) -> Result<String, std::io::Error> {
            Ok("Mock note content".to_string())
        }
        //let mock_editor = |_| Ok("Mock note content".to_string());

        // 3. Call the function with the mock editor.
        let result = create_note(&db, key, mock_editor);
        assert!(result.is_ok());

        // 4. Verify that the note was actually saved in the database.
        let saved_value = db.get(key).unwrap().unwrap();
        assert_eq!(saved_value, "Mock note content".as_bytes());
    }

    #[test]
    fn test_create_note_key_exists() {
        // Setup a temporary db with a pre-existing key.
        let config = Config::new().temporary(true);
        let db = config.open().expect("Failed to open temporary db");
        let key = "existing-key";
        db.insert(key, "old value").unwrap();

        fn mock_editor(_: &str) -> Result<String, std::io::Error> {
            Ok("Mock note content".to_string())
        }
        //let mock_editor = |_| Ok("".to_string()); // Editor won't even be called

        // Call the function and assert that it returns the correct error.
        let result = create_note(&db, key, mock_editor);
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
}