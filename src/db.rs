use crate::config::Config;
use crate::error::AppError;
use crate::note::Note;
use serde_json;
use sled::Db;
use std::path::PathBuf;
use std::{env, fs, str};
use tantivy::{doc, Index, IndexWriter, TantivyDocument, Term};

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
    let mut index_writer: tantivy::IndexWriter<tantivy::TantivyDocument> = index.writer(50_000_000)?;
    let schema = index.schema();
    let key_field = schema.get_field("key")?;

    // For updates, first delete the old document
    let key_term = Term::from_field_text(key_field, &note.key);
    index_writer.delete_term(key_term);

    // Add the new document
    let key = schema.get_field("key")?;
    let title = schema.get_field("title")?;
    let content = schema.get_field("content")?;
    let tags_field = schema.get_field("tags")?;

    let mut doc = doc!(
        key => note.key.clone(),
        title => note.title.clone(),
        content => note.content.clone(),
    );

    // Add each tag as a separate field value
    for tag in &note.tags {
        doc.add_text(tags_field, tag);
    }

    index_writer.add_document(doc)?;

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
            let key_field = index.schema().get_field("key")?;
            let key_term = Term::from_field_text(key_field, key);
            index_writer.delete_term(key_term);
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
