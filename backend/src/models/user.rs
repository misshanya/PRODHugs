use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub hashed_password: String,
    pub role: String,
    pub gender: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    pub hashed_password: String,
    pub gender: Option<String>,
    pub display_name: Option<String>,
    pub tag: Option<String>,
    pub special_tag: Option<String>,
    pub telegram_id: Option<i64>,
    pub banned_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub captcha_type: String,
    pub captcha_cooldown_until: Option<DateTime<Utc>>,
    pub promoted_until: Option<DateTime<Utc>>,
    pub promotion_message: Option<String>,
    pub promotion_bid: i32,
    pub vip_remaining_seconds: i32,
    pub vip_cooldown_until: Option<DateTime<Utc>>,
    pub is_recently_active: bool,
    pub is_telegram_linked: bool,
    pub avg_response_time: Option<f64>,
    pub balance: i32,
}

#[derive(Debug, Clone)]
pub struct AdminUser {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    pub gender: Option<String>,
    pub display_name: Option<String>,
    pub tag: Option<String>,
    pub special_tag: Option<String>,
    pub banned_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub balance: i32,
    pub last_visit_at: Option<DateTime<Utc>>,
    pub captcha_type: String,
    pub captcha_cooldown_until: Option<DateTime<Utc>>,
    pub promoted_until: Option<DateTime<Utc>>,
    pub promotion_message: Option<String>,
    pub promotion_bid: i32,
    pub vip_remaining_seconds: i32,
    pub vip_cooldown_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct AdminStats {
    pub total_users: i64,
    pub banned_users: i64,
}

#[derive(Debug, Clone)]
pub struct BlockedUser {
    pub id: Uuid,
    pub username: String,
    pub gender: Option<String>,
    pub display_name: Option<String>,
    pub tag: Option<String>,
    pub special_tag: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Promotion bid → coin cost; 5 coins per hour.
pub fn promotion_cost(hours: i32) -> i32 {
    hours * 5
}
