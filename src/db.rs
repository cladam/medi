use crate::error::AppError;
use sled::Db;
use std::{env, fs, str};
use std::path::PathBuf;
use anyhow::anyhow;

// Helper function to open the database
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

// Corresponds to `medi new <key>`
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

// Corresponds to `medi get <key>`
pub fn get_note(db: &Db, key: &str) -> Result<String, AppError> {
    let value_ivec = db.get(key)?
        .ok_or_else(|| AppError::KeyNotFound(key.to_string()))?;
    
    Ok(str::from_utf8(&value_ivec)?.to_string())
}

// Corresponds to `medi list`
pub fn list_notes(db: &Db) -> Result<Vec<String>, AppError> {
    let mut keys = Vec::new();
    for result in db.iter() {
        let (key, _) = result.map_err(AppError::from)?;
        let key_str = str::from_utf8(&key).map_err(AppError::Utf8)?;
        keys.push(key_str.to_string());
    }
    Ok(keys)
}

// Corresponds to `medi delete <key>`
pub fn delete_note(db: &Db, key: &str) -> Result<(), AppError> {
    if !db.contains_key(key)? {
        return Err(AppError::KeyNotFound(key.to_string()));
    }
    db.remove(key)?;
    db.flush()?; // Ensure data is saved to disk
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
}