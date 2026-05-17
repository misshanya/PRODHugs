use chrono::{DateTime, Utc};
use uuid::Uuid;

pub const HUG_STATUS_PENDING: &str = "pending";
pub const HUG_STATUS_COMPLETED: &str = "completed";
pub const HUG_STATUS_DECLINED: &str = "declined";
pub const HUG_STATUS_EXPIRED: &str = "expired";
pub const HUG_STATUS_CANCELLED: &str = "cancelled";

pub const HUG_TYPE_STANDARD: &str = "standard";
pub const HUG_TYPE_BEAR: &str = "bear";
pub const HUG_TYPE_GROUP: &str = "group";
pub const HUG_TYPE_WARM: &str = "warm";
pub const HUG_TYPE_SOUL: &str = "soul";

pub const MAX_HUG_SLOTS: i32 = 5;

#[derive(Debug, Clone)]
pub struct Hug {
    pub id: Uuid,
    pub giver_id: Uuid,
    pub receiver_id: Uuid,
    pub status: String,
    pub hug_type: String,
    pub comment: Option<String>,
    pub streak_tier: String,
    pub created_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct HugFeedItem {
    pub id: Uuid,
    pub giver_id: Uuid,
    pub receiver_id: Uuid,
    pub giver_username: String,
    pub receiver_username: String,
    pub giver_gender: Option<String>,
    pub giver_display_name: Option<String>,
    pub receiver_display_name: Option<String>,
    pub hug_type: String,
    pub has_comment: bool,
    pub streak_tier: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct HugActivityItem {
    pub timestamp: DateTime<Utc>,
    pub count: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct MutualHugStats {
    pub total: i64,
    pub given: i64,
    pub received: i64,
}

#[derive(Debug, Clone)]
pub struct HugCooldown {
    pub user_a_id: Uuid,
    pub user_b_id: Uuid,
    pub last_hug_at: DateTime<Utc>,
    pub cooldown_seconds: i32,
    pub decline_cooldown_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct PendingHugInboxItem {
    pub id: Uuid,
    pub giver_id: Uuid,
    pub receiver_id: Uuid,
    pub giver_username: String,
    pub giver_gender: Option<String>,
    pub giver_display_name: Option<String>,
    pub hug_type: String,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct OutgoingPendingHug {
    pub id: Uuid,
    pub giver_id: Uuid,
    pub receiver_id: Uuid,
    pub receiver_username: String,
    pub receiver_gender: Option<String>,
    pub receiver_display_name: Option<String>,
    pub hug_type: String,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct HugDetail {
    pub id: Uuid,
    pub giver_id: Uuid,
    pub receiver_id: Uuid,
    pub giver_username: String,
    pub receiver_username: String,
    pub giver_gender: Option<String>,
    pub giver_display_name: Option<String>,
    pub receiver_display_name: Option<String>,
    pub status: String,
    pub hug_type: String,
    pub comment: Option<String>,
    pub streak_tier: String,
    pub created_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct SlotInfo {
    pub total_slots: i32,
    pub used_slots: i32,
    pub next_slot_cost: Option<i32>,
}

/// Cost for the Nth slot (1-indexed). Slot 1 is free. Slots 2..=5 cost 10, 20, 30, 40.
pub fn slot_cost(slot_number: i32) -> i32 {
    if slot_number <= 1 {
        0
    } else {
        (slot_number - 1) * 10
    }
}

pub fn valid_hug_type(hug_type: &str) -> bool {
    matches!(
        hug_type,
        HUG_TYPE_STANDARD
            | HUG_TYPE_BEAR
            | HUG_TYPE_GROUP
            | HUG_TYPE_WARM
            | HUG_TYPE_SOUL
    )
}
