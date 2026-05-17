use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Announcement {
    pub id: Uuid,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub active: bool,
}
