use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub tag: Option<String>,
    pub special_tag: Option<String>,
    pub role: String,
    pub total_hugs: i32,
    pub hugs_given: i64,
    pub hugs_received: i64,
    pub rank: String,
}

#[derive(Debug, Clone)]
pub struct UserStats {
    pub hugs_given: i64,
    pub hugs_received: i64,
    pub total_hugs: i32,
    pub rank: String,
}

struct RankDef {
    male: &'static str,
    female: &'static str,
    unknown: &'static str,
}

const RANKS: &[(i32, RankDef)] = &[
    (
        1000,
        RankDef {
            male: "Тактильный маньяк",
            female: "Тактильная маньячка",
            unknown: "Тактильный(ая) маньяк(чка)",
        },
    ),
    (
        500,
        RankDef {
            male: "Легенда",
            female: "Легенда",
            unknown: "Легенда",
        },
    ),
    (
        200,
        RankDef {
            male: "Обнимастер",
            female: "Обнимастер",
            unknown: "Обнимастер",
        },
    ),
    (
        50,
        RankDef {
            male: "Тактильный",
            female: "Тактильная",
            unknown: "Тактильный(ая)",
        },
    ),
    (
        10,
        RankDef {
            male: "Неопытный",
            female: "Неопытная",
            unknown: "Неопытный(ая)",
        },
    ),
    (
        0,
        RankDef {
            male: "Нетактильный",
            female: "Нетактильная",
            unknown: "Нетактильный(ая)",
        },
    ),
];

pub fn get_rank(total_hugs: i32, gender: Option<&str>) -> String {
    for (min, def) in RANKS {
        if total_hugs >= *min {
            return pick(gender, def).to_string();
        }
    }
    pick(gender, &RANKS.last().unwrap().1).to_string()
}

fn pick<'a>(gender: Option<&str>, def: &'a RankDef) -> &'a str {
    match gender {
        Some("male") => def.male,
        Some("female") => def.female,
        _ => def.unknown,
    }
}
