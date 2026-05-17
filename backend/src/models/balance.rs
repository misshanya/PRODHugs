use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Balance {
    pub user_id: Uuid,
    pub amount: i32,
    pub updated_at: DateTime<Utc>,
}
