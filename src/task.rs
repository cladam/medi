use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TaskStatus {
    Open,
    Prio,
    Done,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: u64,
    pub note_key: String,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
}
