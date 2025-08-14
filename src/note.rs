use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// This module defines the structure of a Note in the medi application.
// A Note consists of a key, title, tags, content, and timestamps for creation and modification.
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