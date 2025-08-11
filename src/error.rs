use dialoguer::Error as DialoguerError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Sled(#[from] sled::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("I/O error during edit: {0}")]
    Io(#[from] std::io::Error),

    #[error("User input error: {0}")]
    Dialoguer(#[from] DialoguerError),

    #[error("Key '{0}' not found in the database")]
    KeyNotFound(String),

    #[error("Key '{0}' already exists. Use 'edit' to modify it.")]
    KeyExists(String),
}