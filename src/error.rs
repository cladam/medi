use dialoguer::Error as DialoguerError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    #[error("Database error: {0}")]
    Sled(#[from] sled::Error),

    #[error("Regexp error: {0}")]
    Regexp(#[from] regex::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Failed to convert bytes to UTF-8 string: {0}")]
    UTF8Conversion(#[from] std::string::FromUtf8Error),

    #[error("I/O error during edit: {0}")]
    Io(#[from] std::io::Error),

    #[error("User input error: {0}")]
    Dialoguer(#[from] DialoguerError),

    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Key '{0}' not found in the database")]
    KeyNotFound(String),

    #[error("Key '{0}' already exists. Use 'edit' to modify it.")]
    KeyExists(String),

    #[error("Self-update error: {0}")]
    SelfUpdate(#[from] self_update::errors::Error),

    #[error("Search operation failed: {0}")]
    Search(String),

    #[error("Tantivy error: {0}")]
    Tantivy(#[from] tantivy::error::TantivyError),

    #[error("Task with ID '{0}' not found")]
    TaskNotFound(u64),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
