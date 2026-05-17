use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PairStreak {
    pub user_a_id: Uuid,
    pub user_b_id: Uuid,
    pub current_streak: i32,
    pub best_streak: i32,
    pub last_streak_date: Option<NaiveDate>,
    pub a_hugged_today: bool,
    pub b_hugged_today: bool,
    pub today_date: NaiveDate,
}

#[derive(Debug, Clone, Copy)]
pub struct StreakTier {
    pub level: i32,
    pub name: &'static str,
    pub key: &'static str,
    pub min_days: i32,
}

pub const STREAK_TIERS: &[StreakTier] = &[
    StreakTier {
        level: 6,
        name: "Легендарная",
        key: "legendary",
        min_days: 90,
    },
    StreakTier {
        level: 5,
        name: "Обсидиановая",
        key: "obsidian",
        min_days: 60,
    },
    StreakTier {
        level: 4,
        name: "Алмазная",
        key: "diamond",
        min_days: 30,
    },
    StreakTier {
        level: 3,
        name: "Сапфировая",
        key: "sapphire",
        min_days: 21,
    },
    StreakTier {
        level: 2,
        name: "Рубиновая",
        key: "ruby",
        min_days: 14,
    },
    StreakTier {
        level: 1,
        name: "Изумрудная",
        key: "emerald",
        min_days: 7,
    },
    StreakTier {
        level: 0,
        name: "",
        key: "",
        min_days: 0,
    },
];

pub fn compute_streak_tier(streak_days: i32) -> StreakTier {
    for tier in STREAK_TIERS {
        if streak_days >= tier.min_days {
            return *tier;
        }
    }
    STREAK_TIERS[STREAK_TIERS.len() - 1]
}

#[derive(Debug, Clone)]
pub struct StreakInfo {
    pub current_streak: i32,
    pub best_streak: i32,
    pub tier_level: i32,
    pub tier_name: String,
    pub tier_key: String,
    pub next_tier_at: Option<i32>,
    pub a_hugged_today: bool,
    pub b_hugged_today: bool,
}

pub fn compute_streak_info(streak: Option<&PairStreak>) -> StreakInfo {
    let Some(streak) = streak else {
        return StreakInfo {
            current_streak: 0,
            best_streak: 0,
            tier_level: 0,
            tier_name: String::new(),
            tier_key: String::new(),
            next_tier_at: None,
            a_hugged_today: false,
            b_hugged_today: false,
        };
    };
    let tier = compute_streak_tier(streak.current_streak);
    let mut info = StreakInfo {
        current_streak: streak.current_streak,
        best_streak: streak.best_streak,
        tier_level: tier.level,
        tier_name: tier.name.to_string(),
        tier_key: tier.key.to_string(),
        next_tier_at: None,
        a_hugged_today: streak.a_hugged_today,
        b_hugged_today: streak.b_hugged_today,
    };
    for t in STREAK_TIERS.iter().rev() {
        if t.min_days > streak.current_streak {
            info.next_tier_at = Some(t.min_days);
            break;
        }
    }
    info
}

#[derive(Debug, Clone)]
pub struct TopStreakEntry {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub gender: Option<String>,
    pub current_streak: i32,
    pub best_streak: i32,
    pub tier_level: i32,
    pub tier_name: String,
    pub tier_key: String,
}

#[derive(Debug, Clone)]
pub struct StreakCalendarDay {
    pub date: NaiveDate,
    pub hug_count: i64,
    pub completed: bool,
}
