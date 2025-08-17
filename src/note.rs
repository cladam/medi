use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// This module defines the structure of a Note in the medi application.
/// A Note consists of a key, title, tags, content, and timestamps for creation and modification.
#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    pub key: String,
    pub title: String,
    #[serde(default)] // If tags are missing in old data, default to empty Vec
    pub tags: Vec<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// Represents the JSON structure for exporting notes.
/// This structure includes the export date, the count of notes, and a vector of Note objects
#[derive(Serialize)]
pub struct JsonExport {
    pub export_date: DateTime<Utc>,
    pub note_count: usize,
    pub notes: Vec<Note>,
}