use chrono::{DateTime, Utc};
use uuid::Uuid;

pub const MAX_USER_NOTE_LENGTH: usize = 256;

#[derive(Debug, Clone)]
pub struct UserNote {
    pub author_id: Uuid,
    pub target_id: Uuid,
    pub content: String,
    pub updated_at: DateTime<Utc>,
    pub target_username: String,
    pub target_display_name: Option<String>,
}
