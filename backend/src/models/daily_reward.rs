use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DailyReward {
    pub user_id: Uuid,
    pub last_claimed_at: DateTime<Utc>,
    pub streak_days: i32,
}

pub fn today_ymd() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}
