use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PairIntimacy {
    pub user_a_id: Uuid,
    pub user_b_id: Uuid,
    pub raw_score: i32,
    pub last_hug_at: DateTime<Utc>,
    pub last_decay_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct IntimacyTier {
    pub level: i32,
    pub name: &'static str,
    pub min_score: i32,
    pub cooldown_reduction: f64,
    pub bonus_coins: i32,
    pub unlocked_hug_types: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub struct IntimacyInfo {
    pub raw_score: i32,
    pub tier: i32,
    pub tier_name: String,
    pub next_tier_at: Option<i32>,
    pub cooldown_reduction_pct: i32,
    pub available_hug_types: Vec<String>,
    pub bonus_coins: i32,
}

#[derive(Debug, Clone)]
pub struct ConnectionItem {
    pub user_id: Uuid,
    pub username: String,
    pub gender: Option<String>,
    pub display_name: Option<String>,
    pub intimacy: IntimacyInfo,
}

#[derive(Debug, Clone)]
pub struct LeaderboardPairEntry {
    pub user_a_id: Uuid,
    pub user_a_username: String,
    pub user_a_display_name: Option<String>,
    pub user_b_id: Uuid,
    pub user_b_username: String,
    pub user_b_display_name: Option<String>,
    pub raw_score: i32,
    pub tier: i32,
    pub tier_name: String,
}

pub const INTIMACY_TIERS: &[IntimacyTier] = &[
    IntimacyTier {
        level: 0,
        name: "Незнакомцы",
        min_score: 0,
        cooldown_reduction: 0.0,
        bonus_coins: 0,
        unlocked_hug_types: &[crate::models::hug::HUG_TYPE_STANDARD],
    },
    IntimacyTier {
        level: 1,
        name: "Знакомые",
        min_score: 5,
        cooldown_reduction: 0.10,
        bonus_coins: 0,
        unlocked_hug_types: &[crate::models::hug::HUG_TYPE_STANDARD],
    },
    IntimacyTier {
        level: 2,
        name: "Приятели",
        min_score: 15,
        cooldown_reduction: 0.20,
        bonus_coins: 1,
        unlocked_hug_types: &[
            crate::models::hug::HUG_TYPE_STANDARD,
            crate::models::hug::HUG_TYPE_BEAR,
        ],
    },
    IntimacyTier {
        level: 3,
        name: "Друзья",
        min_score: 30,
        cooldown_reduction: 0.30,
        bonus_coins: 1,
        unlocked_hug_types: &[
            crate::models::hug::HUG_TYPE_STANDARD,
            crate::models::hug::HUG_TYPE_BEAR,
            crate::models::hug::HUG_TYPE_GROUP,
        ],
    },
    IntimacyTier {
        level: 4,
        name: "Близкие",
        min_score: 50,
        cooldown_reduction: 0.40,
        bonus_coins: 2,
        unlocked_hug_types: &[
            crate::models::hug::HUG_TYPE_STANDARD,
            crate::models::hug::HUG_TYPE_BEAR,
            crate::models::hug::HUG_TYPE_GROUP,
            crate::models::hug::HUG_TYPE_WARM,
        ],
    },
    IntimacyTier {
        level: 5,
        name: "Родные души",
        min_score: 80,
        cooldown_reduction: 0.50,
        bonus_coins: 2,
        unlocked_hug_types: &[
            crate::models::hug::HUG_TYPE_STANDARD,
            crate::models::hug::HUG_TYPE_BEAR,
            crate::models::hug::HUG_TYPE_GROUP,
            crate::models::hug::HUG_TYPE_WARM,
            crate::models::hug::HUG_TYPE_SOUL,
        ],
    },
];

pub fn compute_tier(raw_score: i32) -> &'static IntimacyTier {
    let mut current = &INTIMACY_TIERS[0];
    for tier in INTIMACY_TIERS {
        if raw_score >= tier.min_score {
            current = tier;
        } else {
            break;
        }
    }
    current
}

pub fn compute_intimacy_info(raw_score: i32) -> IntimacyInfo {
    let tier = compute_tier(raw_score);
    let next_tier_at = INTIMACY_TIERS
        .iter()
        .find(|t| t.level == tier.level + 1)
        .map(|t| t.min_score);
    IntimacyInfo {
        raw_score,
        tier: tier.level,
        tier_name: tier.name.to_string(),
        next_tier_at,
        cooldown_reduction_pct: (tier.cooldown_reduction * 100.0) as i32,
        available_hug_types: tier
            .unlocked_hug_types
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        bonus_coins: tier.bonus_coins,
    }
}

pub fn is_hug_type_unlocked(raw_score: i32, hug_type: &str) -> bool {
    compute_tier(raw_score)
        .unlocked_hug_types
        .iter()
        .any(|s| *s == hug_type)
}
