use std::path::PathBuf;
use crate::error::AppError;
use sled::Db;
use std::{fs, str};

// Helper function to open the database
pub fn open() -> Result<Db, AppError> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find home directory",
        ))
    })?;

    let db_dir = home_dir.join(".medi");
    fs::create_dir_all(&db_dir)?;

    let db_path = db_dir.join("medi_db");
    sled::open(db_path).map_err(AppError::from)
}

// Corresponds to `medi new <key>`
pub fn create_note(db: &Db, key: &str) -> Result<(), AppError> {
    if db.contains_key(key)? {
        return Err(AppError::KeyExists(key.to_string()));
    }

    let value = edit::edit("")?; // Opens blank editor

    // Don't save if the user didn't write anything
    if value.trim().is_empty() {
        println!("Note creation cancelled (empty content).");
        return Ok(());
    }

    db.insert(key, value.as_bytes())?;
    db.flush()?; // Ensure data is saved to disk
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
    // TODO: Get the note from the db.
    // If not found, return AppError::KeyNotFound.
    // Convert the bytes to a String and return it.
    Ok("".to_string()) // Placeholder
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