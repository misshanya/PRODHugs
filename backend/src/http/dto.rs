//! JSON DTOs shared by v1 + v2 handlers.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models;

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telegram_id: Option<i64>,
    pub captcha_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha_cooldown_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promotion_message: Option<String>,
    pub promotion_bid: i32,
    pub vip_remaining_seconds: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vip_cooldown_until: Option<DateTime<Utc>>,
    pub is_recently_active: bool,
    pub balance: i32,
}

impl From<&models::User> for UserDto {
    fn from(u: &models::User) -> Self {
        Self {
            id: u.id,
            username: u.username.clone(),
            role: u.role.clone(),
            gender: u.gender.clone(),
            display_name: u.display_name.clone(),
            tag: u.tag.clone(),
            special_tag: u.special_tag.clone(),
            telegram_id: u.telegram_id,
            captcha_type: u.captcha_type.clone(),
            captcha_cooldown_until: u.captcha_cooldown_until,
            promoted_until: u.promoted_until,
            promotion_message: u.promotion_message.clone(),
            promotion_bid: u.promotion_bid,
            vip_remaining_seconds: u.vip_remaining_seconds,
            vip_cooldown_until: u.vip_cooldown_until,
            is_recently_active: u.is_recently_active,
            balance: u.balance,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserListItemDto {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_tag: Option<String>,
    pub is_telegram_linked: bool,
    pub is_recently_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_response_time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promotion_message: Option<String>,
    pub promotion_bid: i32,
    pub vip_remaining_seconds: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vip_cooldown_until: Option<DateTime<Utc>>,
}

impl From<&models::User> for UserListItemDto {
    fn from(u: &models::User) -> Self {
        Self {
            id: u.id,
            username: u.username.clone(),
            role: u.role.clone(),
            gender: u.gender.clone(),
            display_name: u.display_name.clone(),
            tag: u.tag.clone(),
            special_tag: u.special_tag.clone(),
            is_telegram_linked: u.is_telegram_linked,
            is_recently_active: u.is_recently_active,
            avg_response_time: u.avg_response_time,
            promoted_until: u.promoted_until,
            promotion_message: u.promotion_message.clone(),
            promotion_bid: u.promotion_bid,
            vip_remaining_seconds: u.vip_remaining_seconds,
            vip_cooldown_until: u.vip_cooldown_until,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AdminUserDto {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_tag: Option<String>,
    pub balance: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banned_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_visit_at: Option<DateTime<Utc>>,
    pub captcha_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha_cooldown_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promotion_message: Option<String>,
    pub promotion_bid: i32,
    pub vip_remaining_seconds: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vip_cooldown_until: Option<DateTime<Utc>>,
}

impl From<&models::AdminUser> for AdminUserDto {
    fn from(u: &models::AdminUser) -> Self {
        Self {
            id: u.id,
            username: u.username.clone(),
            role: u.role.clone(),
            gender: u.gender.clone(),
            display_name: u.display_name.clone(),
            tag: u.tag.clone(),
            special_tag: u.special_tag.clone(),
            balance: u.balance,
            banned_at: u.banned_at,
            created_at: u.created_at,
            last_visit_at: u.last_visit_at,
            captcha_type: u.captcha_type.clone(),
            captcha_cooldown_until: u.captcha_cooldown_until,
            promoted_until: u.promoted_until,
            promotion_message: u.promotion_message.clone(),
            promotion_bid: u.promotion_bid,
            vip_remaining_seconds: u.vip_remaining_seconds,
            vip_cooldown_until: u.vip_cooldown_until,
        }
    }
}

impl From<&models::User> for AdminUserDto {
    fn from(u: &models::User) -> Self {
        Self {
            id: u.id,
            username: u.username.clone(),
            role: u.role.clone(),
            gender: u.gender.clone(),
            display_name: u.display_name.clone(),
            tag: u.tag.clone(),
            special_tag: u.special_tag.clone(),
            balance: u.balance,
            banned_at: u.banned_at,
            created_at: u.created_at,
            last_visit_at: None,
            captcha_type: u.captcha_type.clone(),
            captcha_cooldown_until: u.captcha_cooldown_until,
            promoted_until: u.promoted_until,
            promotion_message: u.promotion_message.clone(),
            promotion_bid: u.promotion_bid,
            vip_remaining_seconds: u.vip_remaining_seconds,
            vip_cooldown_until: u.vip_cooldown_until,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserDto,
}

#[derive(Debug, Serialize)]
pub struct HugFeedItemDto {
    pub id: Uuid,
    pub giver_id: Uuid,
    pub receiver_id: Uuid,
    pub giver_username: String,
    pub receiver_username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub giver_gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub giver_display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_display_name: Option<String>,
    pub hug_type: String,
    pub has_comment: bool,
    pub streak_tier: String,
    pub created_at: DateTime<Utc>,
}

impl From<&models::HugFeedItem> for HugFeedItemDto {
    fn from(h: &models::HugFeedItem) -> Self {
        Self {
            id: h.id,
            giver_id: h.giver_id,
            receiver_id: h.receiver_id,
            giver_username: h.giver_username.clone(),
            receiver_username: h.receiver_username.clone(),
            giver_gender: h.giver_gender.clone(),
            giver_display_name: h.giver_display_name.clone(),
            receiver_display_name: h.receiver_display_name.clone(),
            hug_type: h.hug_type.clone(),
            has_comment: h.has_comment,
            streak_tier: h.streak_tier.clone(),
            created_at: h.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PendingHugInboxItemDto {
    pub id: Uuid,
    pub giver_id: Uuid,
    pub receiver_id: Uuid,
    pub giver_username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub giver_gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub giver_display_name: Option<String>,
    pub hug_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<&models::PendingHugInboxItem> for PendingHugInboxItemDto {
    fn from(p: &models::PendingHugInboxItem) -> Self {
        Self {
            id: p.id,
            giver_id: p.giver_id,
            receiver_id: p.receiver_id,
            giver_username: p.giver_username.clone(),
            giver_gender: p.giver_gender.clone(),
            giver_display_name: p.giver_display_name.clone(),
            hug_type: p.hug_type.clone(),
            comment: p.comment.clone(),
            created_at: p.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OutgoingPendingHugDto {
    pub id: Uuid,
    pub giver_id: Uuid,
    pub receiver_id: Uuid,
    pub receiver_username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_display_name: Option<String>,
    pub hug_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<&models::OutgoingPendingHug> for OutgoingPendingHugDto {
    fn from(p: &models::OutgoingPendingHug) -> Self {
        Self {
            id: p.id,
            giver_id: p.giver_id,
            receiver_id: p.receiver_id,
            receiver_username: p.receiver_username.clone(),
            receiver_gender: p.receiver_gender.clone(),
            receiver_display_name: p.receiver_display_name.clone(),
            hug_type: p.hug_type.clone(),
            comment: p.comment.clone(),
            created_at: p.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SlotInfoDto {
    pub total_slots: i32,
    pub used_slots: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_slot_cost: Option<i32>,
}

impl From<&models::SlotInfo> for SlotInfoDto {
    fn from(s: &models::SlotInfo) -> Self {
        Self {
            total_slots: s.total_slots,
            used_slots: s.used_slots,
            next_slot_cost: s.next_slot_cost,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct IntimacyInfoDto {
    pub raw_score: i32,
    pub tier: i32,
    pub tier_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_tier_at: Option<i32>,
    pub cooldown_reduction_pct: i32,
    pub available_hug_types: Vec<String>,
    pub bonus_coins: i32,
}

impl From<&models::IntimacyInfo> for IntimacyInfoDto {
    fn from(i: &models::IntimacyInfo) -> Self {
        Self {
            raw_score: i.raw_score,
            tier: i.tier,
            tier_name: i.tier_name.clone(),
            next_tier_at: i.next_tier_at,
            cooldown_reduction_pct: i.cooldown_reduction_pct,
            available_hug_types: i.available_hug_types.clone(),
            bonus_coins: i.bonus_coins,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StreakInfoDto {
    pub current_streak: i32,
    pub best_streak: i32,
    pub tier_level: i32,
    pub tier_name: String,
    pub tier_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_tier_at: Option<i32>,
    pub a_hugged_today: bool,
    pub b_hugged_today: bool,
}

impl From<&models::StreakInfo> for StreakInfoDto {
    fn from(s: &models::StreakInfo) -> Self {
        Self {
            current_streak: s.current_streak,
            best_streak: s.best_streak,
            tier_level: s.tier_level,
            tier_name: s.tier_name.clone(),
            tier_key: s.tier_key.clone(),
            next_tier_at: s.next_tier_at,
            a_hugged_today: s.a_hugged_today,
            b_hugged_today: s.b_hugged_today,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StreakCalendarDayDto {
    pub date: NaiveDate,
    pub hug_count: i64,
    pub completed: bool,
}

impl From<&models::StreakCalendarDay> for StreakCalendarDayDto {
    fn from(s: &models::StreakCalendarDay) -> Self {
        Self {
            date: s.date,
            hug_count: s.hug_count,
            completed: s.completed,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LeaderboardEntryDto {
    pub user_id: Uuid,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_tag: Option<String>,
    pub total_hugs: i32,
    pub hugs_given: i64,
    pub hugs_received: i64,
    pub rank: String,
}

impl From<&models::LeaderboardEntry> for LeaderboardEntryDto {
    fn from(e: &models::LeaderboardEntry) -> Self {
        Self {
            user_id: e.user_id,
            username: e.username.clone(),
            display_name: e.display_name.clone(),
            tag: e.tag.clone(),
            special_tag: e.special_tag.clone(),
            total_hugs: e.total_hugs,
            hugs_given: e.hugs_given,
            hugs_received: e.hugs_received,
            rank: e.rank.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct IntimacyLeaderboardEntryDto {
    pub user_a_id: Uuid,
    pub user_a_username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_a_display_name: Option<String>,
    pub user_b_id: Uuid,
    pub user_b_username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_b_display_name: Option<String>,
    pub raw_score: i32,
    pub tier: i32,
    pub tier_name: String,
}

impl From<&models::LeaderboardPairEntry> for IntimacyLeaderboardEntryDto {
    fn from(e: &models::LeaderboardPairEntry) -> Self {
        Self {
            user_a_id: e.user_a_id,
            user_a_username: e.user_a_username.clone(),
            user_a_display_name: e.user_a_display_name.clone(),
            user_b_id: e.user_b_id,
            user_b_username: e.user_b_username.clone(),
            user_b_display_name: e.user_b_display_name.clone(),
            raw_score: e.raw_score,
            tier: e.tier,
            tier_name: e.tier_name.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConnectionItemDto {
    pub user_id: Uuid,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub intimacy: IntimacyInfoDto,
}

impl From<&models::ConnectionItem> for ConnectionItemDto {
    fn from(c: &models::ConnectionItem) -> Self {
        Self {
            user_id: c.user_id,
            username: c.username.clone(),
            gender: c.gender.clone(),
            display_name: c.display_name.clone(),
            intimacy: IntimacyInfoDto::from(&c.intimacy),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BlockedUserDto {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_tag: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<&models::BlockedUser> for BlockedUserDto {
    fn from(u: &models::BlockedUser) -> Self {
        Self {
            id: u.id,
            username: u.username.clone(),
            gender: u.gender.clone(),
            display_name: u.display_name.clone(),
            tag: u.tag.clone(),
            special_tag: u.special_tag.clone(),
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserNoteDto {
    pub target_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_display_name: Option<String>,
    pub content: String,
    pub updated_at: DateTime<Utc>,
}

impl From<&models::UserNote> for UserNoteDto {
    fn from(n: &models::UserNote) -> Self {
        Self {
            target_id: n.target_id,
            target_username: (!n.target_username.is_empty()).then(|| n.target_username.clone()),
            target_display_name: n.target_display_name.clone(),
            content: n.content.clone(),
            updated_at: n.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TopStreakEntryDto {
    pub user_id: Uuid,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    pub current_streak: i32,
    pub best_streak: i32,
    pub tier_level: i32,
    pub tier_name: String,
    pub tier_key: String,
}

impl From<&models::TopStreakEntry> for TopStreakEntryDto {
    fn from(e: &models::TopStreakEntry) -> Self {
        Self {
            user_id: e.user_id,
            username: e.username.clone(),
            display_name: e.display_name.clone(),
            gender: e.gender.clone(),
            current_streak: e.current_streak,
            best_streak: e.best_streak,
            tier_level: e.tier_level,
            tier_name: e.tier_name.clone(),
            tier_key: e.tier_key.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HugActivityItemDto {
    pub timestamp: DateTime<Utc>,
    pub count: i64,
}

impl From<&models::HugActivityItem> for HugActivityItemDto {
    fn from(h: &models::HugActivityItem) -> Self {
        Self {
            timestamp: h.timestamp,
            count: h.count,
        }
    }
}

/// WebSocket-specific helpers (different field naming for `created_at`).
pub mod ws {
    use chrono::SecondsFormat;
    use serde::Serialize;

    use crate::models;

    #[derive(Serialize)]
    pub struct FeedItem<'a> {
        pub id: String,
        pub giver_id: String,
        pub receiver_id: String,
        pub giver_username: &'a str,
        pub receiver_username: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub giver_gender: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub giver_display_name: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub receiver_display_name: Option<&'a str>,
        pub hug_type: &'a str,
        pub has_comment: bool,
        pub streak_tier: &'a str,
        pub created_at: String,
    }

    #[derive(Serialize)]
    pub struct PendingInbox<'a> {
        pub id: String,
        pub giver_id: String,
        pub receiver_id: String,
        pub giver_username: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub giver_gender: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub giver_display_name: Option<&'a str>,
        pub hug_type: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub comment: Option<&'a str>,
        pub created_at: String,
    }

    pub fn feed_dto(item: &models::HugFeedItem) -> FeedItem<'_> {
        FeedItem {
            id: item.id.to_string(),
            giver_id: item.giver_id.to_string(),
            receiver_id: item.receiver_id.to_string(),
            giver_username: &item.giver_username,
            receiver_username: &item.receiver_username,
            giver_gender: item.giver_gender.as_deref(),
            giver_display_name: item.giver_display_name.as_deref(),
            receiver_display_name: item.receiver_display_name.as_deref(),
            hug_type: &item.hug_type,
            has_comment: item.has_comment,
            streak_tier: &item.streak_tier,
            created_at: item.created_at.to_rfc3339_opts(SecondsFormat::Secs, true),
        }
    }

    pub fn pending_dto(item: &models::PendingHugInboxItem) -> PendingInbox<'_> {
        PendingInbox {
            id: item.id.to_string(),
            giver_id: item.giver_id.to_string(),
            receiver_id: item.receiver_id.to_string(),
            giver_username: &item.giver_username,
            giver_gender: item.giver_gender.as_deref(),
            giver_display_name: item.giver_display_name.as_deref(),
            hug_type: &item.hug_type,
            comment: item.comment.as_deref(),
            created_at: item.created_at.to_rfc3339_opts(SecondsFormat::Secs, true),
        }
    }
}
