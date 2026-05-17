use chrono::{DateTime, Duration, NaiveDate, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{
    compute_streak_tier, get_rank, Hug, HugActivityItem, HugCooldown, HugDetail, HugFeedItem,
    LeaderboardEntry, MutualHugStats, OutgoingPendingHug, PairStreak, PendingHugInboxItem,
    StreakCalendarDay, TopStreakEntry, UserStats,
};

#[derive(sqlx::FromRow)]
struct HugRow {
    id: Uuid,
    giver_id: Uuid,
    receiver_id: Uuid,
    status: String,
    hug_type: String,
    comment: Option<String>,
    streak_tier: String,
    created_at: DateTime<Utc>,
    accepted_at: Option<DateTime<Utc>>,
}

impl From<HugRow> for Hug {
    fn from(r: HugRow) -> Self {
        Hug {
            id: r.id,
            giver_id: r.giver_id,
            receiver_id: r.receiver_id,
            status: r.status,
            hug_type: r.hug_type,
            comment: r.comment,
            streak_tier: r.streak_tier,
            created_at: r.created_at,
            accepted_at: r.accepted_at,
        }
    }
}

pub async fn insert<'e, E>(
    exec: E,
    giver_id: Uuid,
    receiver_id: Uuid,
    status: &str,
    hug_type: &str,
    comment: Option<&str>,
) -> AppResult<Hug>
where
    E: PgExecutor<'e>,
{
    let row: HugRow = sqlx::query_as(
        "INSERT INTO hugs (giver_id, receiver_id, status, hug_type, comment) VALUES ($1, $2, $3, $4, $5) RETURNING id, giver_id, receiver_id, status, hug_type, comment, streak_tier, created_at, accepted_at",
    )
    .bind(giver_id)
    .bind(receiver_id)
    .bind(status)
    .bind(hug_type)
    .bind(comment)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn accept<'e, E>(
    exec: E,
    id: Uuid,
    receiver_id: Uuid,
    streak_tier: &str,
) -> AppResult<Option<Hug>>
where
    E: PgExecutor<'e>,
{
    let row: Option<HugRow> = sqlx::query_as(
        "UPDATE hugs SET status = 'completed', accepted_at = now(), streak_tier = $3 \
         WHERE id = $1 AND receiver_id = $2 AND status = 'pending' \
         RETURNING id, giver_id, receiver_id, status, hug_type, comment, streak_tier, created_at, accepted_at",
    )
    .bind(id)
    .bind(receiver_id)
    .bind(streak_tier)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn decline<'e, E>(exec: E, id: Uuid, receiver_id: Uuid) -> AppResult<Option<Hug>>
where
    E: PgExecutor<'e>,
{
    let row: Option<HugRow> = sqlx::query_as(
        "UPDATE hugs SET status = 'declined' \
         WHERE id = $1 AND receiver_id = $2 AND status = 'pending' \
         RETURNING id, giver_id, receiver_id, status, hug_type, comment, streak_tier, created_at, accepted_at",
    )
    .bind(id)
    .bind(receiver_id)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn cancel<'e, E>(exec: E, id: Uuid, giver_id: Uuid) -> AppResult<Option<Hug>>
where
    E: PgExecutor<'e>,
{
    let row: Option<HugRow> = sqlx::query_as(
        "UPDATE hugs SET status = 'cancelled' \
         WHERE id = $1 AND giver_id = $2 AND status = 'pending' \
         RETURNING id, giver_id, receiver_id, status, hug_type, comment, streak_tier, created_at, accepted_at",
    )
    .bind(id)
    .bind(giver_id)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn get_by_id<'e, E>(exec: E, id: Uuid) -> AppResult<Option<Hug>>
where
    E: PgExecutor<'e>,
{
    let row: Option<HugRow> = sqlx::query_as(
        "SELECT id, giver_id, receiver_id, status, hug_type, comment, streak_tier, created_at, accepted_at FROM hugs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

#[derive(sqlx::FromRow)]
struct PendingInbox {
    id: Uuid,
    giver_id: Uuid,
    receiver_id: Uuid,
    hug_type: String,
    comment: Option<String>,
    created_at: DateTime<Utc>,
    giver_username: String,
    giver_gender: Option<String>,
    giver_display_name: Option<String>,
}

pub async fn pending_inbox<'e, E>(exec: E, user_id: Uuid) -> AppResult<Vec<PendingHugInboxItem>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<PendingInbox> = sqlx::query_as(
        "SELECT h.id, h.giver_id, h.receiver_id, h.hug_type, h.comment, h.created_at, \
                g.username AS giver_username, g.gender AS giver_gender, g.display_name AS giver_display_name \
         FROM hugs h JOIN users g ON g.id = h.giver_id \
         WHERE h.receiver_id = $1 AND h.status = 'pending' AND h.created_at > now() - INTERVAL '24 hours' \
         ORDER BY h.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| PendingHugInboxItem {
            id: r.id,
            giver_id: r.giver_id,
            receiver_id: r.receiver_id,
            giver_username: r.giver_username,
            giver_gender: r.giver_gender,
            giver_display_name: r.giver_display_name,
            hug_type: r.hug_type,
            comment: r.comment,
            created_at: r.created_at,
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct PendingOutgoing {
    id: Uuid,
    giver_id: Uuid,
    receiver_id: Uuid,
    hug_type: String,
    comment: Option<String>,
    created_at: DateTime<Utc>,
    receiver_username: String,
    receiver_gender: Option<String>,
    receiver_display_name: Option<String>,
}

pub async fn outgoing_pending<'e, E>(exec: E, user_id: Uuid) -> AppResult<Vec<OutgoingPendingHug>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<PendingOutgoing> = sqlx::query_as(
        "SELECT h.id, h.giver_id, h.receiver_id, h.hug_type, h.comment, h.created_at, \
                r.username AS receiver_username, r.gender AS receiver_gender, r.display_name AS receiver_display_name \
         FROM hugs h JOIN users r ON r.id = h.receiver_id \
         WHERE h.giver_id = $1 AND h.status = 'pending' AND h.created_at > now() - INTERVAL '24 hours' \
         ORDER BY h.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| OutgoingPendingHug {
            id: r.id,
            giver_id: r.giver_id,
            receiver_id: r.receiver_id,
            receiver_username: r.receiver_username,
            receiver_gender: r.receiver_gender,
            receiver_display_name: r.receiver_display_name,
            hug_type: r.hug_type,
            comment: r.comment,
            created_at: r.created_at,
        })
        .collect())
}

pub async fn count_pending<'e, E>(exec: E, user_id: Uuid) -> AppResult<i64>
where
    E: PgExecutor<'e>,
{
    let (c,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM hugs WHERE receiver_id = $1 AND status = 'pending' AND created_at > now() - INTERVAL '24 hours'",
    )
    .bind(user_id)
    .fetch_one(exec)
    .await?;
    Ok(c)
}

pub async fn check_suggest_eligibility<'e, E>(
    exec: E,
    giver_id: Uuid,
    receiver_id: Uuid,
) -> AppResult<(i32, bool, bool)>
where
    E: PgExecutor<'e>,
{
    let (out, pair, reverse): (i32, bool, bool) = sqlx::query_as(
        "SELECT \
            (SELECT COUNT(*) FROM hugs WHERE giver_id = $1::uuid AND status = 'pending' AND created_at > now() - INTERVAL '24 hours')::int AS outgoing_count, \
            EXISTS(SELECT 1 FROM hugs WHERE giver_id = $1::uuid AND receiver_id = $2::uuid AND status = 'pending' AND created_at > now() - INTERVAL '24 hours') AS pair_pending, \
            EXISTS(SELECT 1 FROM hugs WHERE giver_id = $2::uuid AND receiver_id = $1::uuid AND status = 'pending' AND created_at > now() - INTERVAL '24 hours') AS reverse_pending",
    )
    .bind(giver_id)
    .bind(receiver_id)
    .fetch_one(exec)
    .await?;
    Ok((out, pair, reverse))
}

pub async fn expire_pending<'e, E>(exec: E) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("UPDATE hugs SET status = 'expired' WHERE status = 'pending' AND created_at <= now() - INTERVAL '24 hours'")
        .execute(exec)
        .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct ListRow {
    id: Uuid,
    giver_id: Uuid,
    receiver_id: Uuid,
    created_at: DateTime<Utc>,
    hug_type: String,
    has_comment: bool,
    streak_tier: String,
    giver_username: String,
    receiver_username: String,
    giver_gender: Option<String>,
    giver_display_name: Option<String>,
    receiver_display_name: Option<String>,
}

impl From<ListRow> for HugFeedItem {
    fn from(r: ListRow) -> Self {
        HugFeedItem {
            id: r.id,
            giver_id: r.giver_id,
            receiver_id: r.receiver_id,
            giver_username: r.giver_username,
            receiver_username: r.receiver_username,
            giver_gender: r.giver_gender,
            giver_display_name: r.giver_display_name,
            receiver_display_name: r.receiver_display_name,
            hug_type: r.hug_type,
            has_comment: r.has_comment,
            streak_tier: r.streak_tier,
            created_at: r.created_at,
        }
    }
}

pub async fn list_for_user<'e, E>(
    exec: E,
    user_id: Uuid,
    limit: i32,
    offset: i32,
) -> AppResult<Vec<HugFeedItem>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<ListRow> = sqlx::query_as(
        "SELECT h.id, h.giver_id, h.receiver_id, COALESCE(h.accepted_at, h.created_at) AS created_at, \
                h.hug_type, (h.comment IS NOT NULL)::bool AS has_comment, h.streak_tier, \
                g.username AS giver_username, r.username AS receiver_username, \
                g.gender AS giver_gender, g.display_name AS giver_display_name, r.display_name AS receiver_display_name \
         FROM hugs h JOIN users g ON g.id = h.giver_id JOIN users r ON r.id = h.receiver_id \
         WHERE (h.giver_id = $1::uuid OR h.receiver_id = $1::uuid) AND h.status = 'completed' \
         ORDER BY COALESCE(h.accepted_at, h.created_at) DESC LIMIT $2 OFFSET $3",
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(exec)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn recent_feed<'e, E>(
    exec: E,
    limit: i32,
    offset: i32,
) -> AppResult<Vec<HugFeedItem>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<ListRow> = sqlx::query_as(
        "SELECT h.id, h.giver_id, h.receiver_id, COALESCE(h.accepted_at, h.created_at) AS created_at, \
                h.hug_type, (h.comment IS NOT NULL)::bool AS has_comment, h.streak_tier, \
                g.username AS giver_username, r.username AS receiver_username, \
                g.gender AS giver_gender, g.display_name AS giver_display_name, r.display_name AS receiver_display_name \
         FROM hugs h JOIN users g ON g.id = h.giver_id JOIN users r ON r.id = h.receiver_id \
         WHERE h.status = 'completed' \
         ORDER BY COALESCE(h.accepted_at, h.created_at) DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(exec)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

#[derive(sqlx::FromRow)]
struct ActivityRow {
    bucket_time: DateTime<Utc>,
    hug_count: i64,
}

pub async fn activity<'e, E>(exec: E) -> AppResult<Vec<HugActivityItem>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<ActivityRow> = sqlx::query_as(
        "SELECT bucket::timestamptz AS bucket_time, COALESCE(COUNT(h.id), 0)::bigint AS hug_count \
         FROM generate_series(DATE_TRUNC('hour', NOW() - INTERVAL '23 hours'), DATE_TRUNC('hour', NOW()), '1 hour'::interval) AS bucket \
         LEFT JOIN hugs h ON h.created_at >= bucket AND h.created_at < bucket + '1 hour'::interval AND h.status = 'completed' \
         GROUP BY bucket ORDER BY bucket",
    )
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| HugActivityItem {
            timestamp: r.bucket_time,
            count: r.hug_count,
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct LbRow {
    id: Uuid,
    username: String,
    role: String,
    gender: Option<String>,
    display_name: Option<String>,
    tag: Option<String>,
    special_tag: Option<String>,
    total_hugs: i64,
    hugs_given: i64,
    hugs_received: i64,
}

pub async fn leaderboard<'e, E>(
    exec: E,
    limit: i32,
    offset: i32,
) -> AppResult<Vec<LeaderboardEntry>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<LbRow> = sqlx::query_as(
        "SELECT u.id, u.username, u.role, u.gender, u.display_name, u.tag, u.special_tag, \
                COALESCE(given.cnt, 0) + COALESCE(received.cnt, 0) AS total_hugs, \
                COALESCE(given.cnt, 0) AS hugs_given, COALESCE(received.cnt, 0) AS hugs_received \
         FROM users u \
         LEFT JOIN (SELECT giver_id, COUNT(*) AS cnt FROM hugs WHERE status = 'completed' GROUP BY giver_id) given ON given.giver_id = u.id \
         LEFT JOIN (SELECT receiver_id, COUNT(*) AS cnt FROM hugs WHERE status = 'completed' GROUP BY receiver_id) received ON received.receiver_id = u.id \
         WHERE u.banned_at IS NULL \
         ORDER BY total_hugs DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let total = r.total_hugs as i32;
            LeaderboardEntry {
                user_id: r.id,
                username: r.username,
                display_name: r.display_name,
                tag: r.tag,
                special_tag: r.special_tag,
                role: r.role,
                total_hugs: total,
                hugs_given: r.hugs_given,
                hugs_received: r.hugs_received,
                rank: get_rank(total, r.gender.as_deref()),
            }
        })
        .collect())
}

pub async fn user_stats<'e, E>(
    exec: E,
    user_id: Uuid,
    gender: Option<&str>,
) -> AppResult<UserStats>
where
    E: PgExecutor<'e>,
{
    let (given, received, total): (i64, i64, i64) = sqlx::query_as(
        "SELECT \
            COUNT(*) FILTER (WHERE giver_id = $1::uuid)::bigint AS hugs_given, \
            COUNT(*) FILTER (WHERE receiver_id = $1::uuid)::bigint AS hugs_received, \
            COUNT(*)::bigint AS total_hugs \
         FROM hugs WHERE (giver_id = $1::uuid OR receiver_id = $1::uuid) AND status = 'completed'",
    )
    .bind(user_id)
    .fetch_one(exec)
    .await?;
    Ok(UserStats {
        hugs_given: given,
        hugs_received: received,
        total_hugs: total as i32,
        rank: get_rank(total as i32, gender),
    })
}

pub async fn count_mutual<'e, E>(
    exec: E,
    user_a: Uuid,
    user_b: Uuid,
) -> AppResult<MutualHugStats>
where
    E: PgExecutor<'e>,
{
    let (total, given, received): (i64, i64, i64) = sqlx::query_as(
        "SELECT \
            COUNT(*)::bigint AS mutual_total, \
            COUNT(*) FILTER (WHERE giver_id = $1 AND receiver_id = $2)::bigint AS mutual_given, \
            COUNT(*) FILTER (WHERE giver_id = $2 AND receiver_id = $1)::bigint AS mutual_received \
         FROM hugs WHERE ((giver_id = $1 AND receiver_id = $2) OR (giver_id = $2 AND receiver_id = $1)) AND status = 'completed'",
    )
    .bind(user_a)
    .bind(user_b)
    .fetch_one(exec)
    .await?;
    Ok(MutualHugStats {
        total,
        given,
        received,
    })
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct DetailRow {
    id: Uuid,
    giver_id: Uuid,
    receiver_id: Uuid,
    status: String,
    hug_type: String,
    comment: Option<String>,
    streak_tier: String,
    created_at: DateTime<Utc>,
    accepted_at: Option<DateTime<Utc>>,
    giver_username: String,
    giver_gender: Option<String>,
    giver_display_name: Option<String>,
    receiver_username: String,
    receiver_gender: Option<String>,
    receiver_display_name: Option<String>,
}

pub async fn detail<'e, E>(exec: E, id: Uuid) -> AppResult<Option<HugDetail>>
where
    E: PgExecutor<'e>,
{
    let _ = (); // suppress unused warnings on receiver_gender below
    let row: Option<DetailRow> = sqlx::query_as(
        "SELECT h.id, h.giver_id, h.receiver_id, h.status, h.hug_type, h.comment, h.streak_tier, h.created_at, h.accepted_at, \
                g.username AS giver_username, g.gender AS giver_gender, g.display_name AS giver_display_name, \
                r.username AS receiver_username, r.gender AS receiver_gender, r.display_name AS receiver_display_name \
         FROM hugs h JOIN users g ON g.id = h.giver_id JOIN users r ON r.id = h.receiver_id WHERE h.id = $1",
    )
    .bind(id)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(|r| HugDetail {
        id: r.id,
        giver_id: r.giver_id,
        receiver_id: r.receiver_id,
        giver_username: r.giver_username,
        receiver_username: r.receiver_username,
        giver_gender: r.giver_gender,
        giver_display_name: r.giver_display_name,
        receiver_display_name: r.receiver_display_name,
        status: r.status,
        hug_type: r.hug_type,
        comment: r.comment,
        streak_tier: r.streak_tier,
        created_at: r.created_at,
        accepted_at: r.accepted_at,
    }))
}

// ── cooldowns ──────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct CooldownRow {
    user_a_id: Uuid,
    user_b_id: Uuid,
    last_hug_at: DateTime<Utc>,
    cooldown_seconds: i32,
    decline_cooldown_until: Option<DateTime<Utc>>,
}

impl From<CooldownRow> for HugCooldown {
    fn from(r: CooldownRow) -> Self {
        HugCooldown {
            user_a_id: r.user_a_id,
            user_b_id: r.user_b_id,
            last_hug_at: r.last_hug_at,
            cooldown_seconds: r.cooldown_seconds,
            decline_cooldown_until: r.decline_cooldown_until,
        }
    }
}

pub async fn get_cooldown<'e, E>(
    exec: E,
    a: Uuid,
    b: Uuid,
) -> AppResult<Option<HugCooldown>>
where
    E: PgExecutor<'e>,
{
    let row: Option<CooldownRow> = sqlx::query_as(
        "SELECT user_a_id, user_b_id, last_hug_at, cooldown_seconds, decline_cooldown_until \
         FROM hug_cooldowns WHERE user_a_id = LEAST($1::uuid, $2::uuid) AND user_b_id = GREATEST($1::uuid, $2::uuid)",
    )
    .bind(a)
    .bind(b)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn upsert_cooldown<'e, E>(
    exec: E,
    a: Uuid,
    b: Uuid,
    cooldown_seconds: i32,
) -> AppResult<HugCooldown>
where
    E: PgExecutor<'e>,
{
    let row: CooldownRow = sqlx::query_as(
        "INSERT INTO hug_cooldowns (user_a_id, user_b_id, cooldown_seconds) \
         VALUES (LEAST($1::uuid, $2::uuid), GREATEST($1::uuid, $2::uuid), $3) \
         ON CONFLICT (user_a_id, user_b_id) DO UPDATE SET last_hug_at = now() \
         RETURNING user_a_id, user_b_id, last_hug_at, cooldown_seconds, decline_cooldown_until",
    )
    .bind(a)
    .bind(b)
    .bind(cooldown_seconds)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn reduce_cooldown<'e, E>(
    exec: E,
    a: Uuid,
    b: Uuid,
    reduction: i32,
) -> AppResult<Option<HugCooldown>>
where
    E: PgExecutor<'e>,
{
    let row: Option<CooldownRow> = sqlx::query_as(
        "UPDATE hug_cooldowns SET cooldown_seconds = GREATEST(cooldown_seconds - $3, 300) \
         WHERE user_a_id = LEAST($1::uuid, $2::uuid) AND user_b_id = GREATEST($1::uuid, $2::uuid) \
         RETURNING user_a_id, user_b_id, last_hug_at, cooldown_seconds, decline_cooldown_until",
    )
    .bind(a)
    .bind(b)
    .bind(reduction)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn set_decline_cooldown<'e, E>(
    exec: E,
    a: Uuid,
    b: Uuid,
    until: DateTime<Utc>,
) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO hug_cooldowns (user_a_id, user_b_id, decline_cooldown_until, cooldown_seconds, last_hug_at) \
         VALUES (LEAST($1::uuid, $2::uuid), GREATEST($1::uuid, $2::uuid), $3, 3600, '2000-01-01'::timestamptz) \
         ON CONFLICT (user_a_id, user_b_id) DO UPDATE SET decline_cooldown_until = $3",
    )
    .bind(a)
    .bind(b)
    .bind(until)
    .execute(exec)
    .await?;
    Ok(())
}

// ── pair streaks ───────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct StreakRow {
    user_a_id: Uuid,
    user_b_id: Uuid,
    current_streak: i32,
    best_streak: i32,
    last_streak_date: Option<NaiveDate>,
    a_hugged_today: bool,
    b_hugged_today: bool,
    today_date: NaiveDate,
}

impl From<StreakRow> for PairStreak {
    fn from(r: StreakRow) -> Self {
        PairStreak {
            user_a_id: r.user_a_id,
            user_b_id: r.user_b_id,
            current_streak: r.current_streak,
            best_streak: r.best_streak,
            last_streak_date: r.last_streak_date,
            a_hugged_today: r.a_hugged_today,
            b_hugged_today: r.b_hugged_today,
            today_date: r.today_date,
        }
    }
}

pub async fn get_pair_streak<'e, E>(
    exec: E,
    a: Uuid,
    b: Uuid,
) -> AppResult<Option<PairStreak>>
where
    E: PgExecutor<'e>,
{
    let row: Option<StreakRow> = sqlx::query_as(
        "SELECT user_a_id, user_b_id, current_streak, best_streak, last_streak_date, a_hugged_today, b_hugged_today, today_date \
         FROM pair_streaks WHERE user_a_id = LEAST($1::uuid, $2::uuid) AND user_b_id = GREATEST($1::uuid, $2::uuid)",
    )
    .bind(a)
    .bind(b)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn upsert_pair_streak<'e, E>(
    exec: E,
    streak: &PairStreak,
) -> AppResult<PairStreak>
where
    E: PgExecutor<'e>,
{
    let row: StreakRow = sqlx::query_as(
        "INSERT INTO pair_streaks (user_a_id, user_b_id, current_streak, best_streak, last_streak_date, a_hugged_today, b_hugged_today, today_date) \
         VALUES (LEAST($1::uuid, $2::uuid), GREATEST($1::uuid, $2::uuid), $3, $4, $5, $6, $7, $8) \
         ON CONFLICT (user_a_id, user_b_id) DO UPDATE \
         SET current_streak = $3, best_streak = $4, last_streak_date = $5, a_hugged_today = $6, b_hugged_today = $7, today_date = $8 \
         RETURNING user_a_id, user_b_id, current_streak, best_streak, last_streak_date, a_hugged_today, b_hugged_today, today_date",
    )
    .bind(streak.user_a_id)
    .bind(streak.user_b_id)
    .bind(streak.current_streak)
    .bind(streak.best_streak)
    .bind(streak.last_streak_date)
    .bind(streak.a_hugged_today)
    .bind(streak.b_hugged_today)
    .bind(streak.today_date)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

#[derive(sqlx::FromRow)]
struct TopStreakRow {
    user_a_id: Uuid,
    user_b_id: Uuid,
    current_streak: i32,
    best_streak: i32,
    last_streak_date: Option<NaiveDate>,
    a_hugged_today: bool,
    b_hugged_today: bool,
    today_date: NaiveDate,
    other_username: String,
    other_display_name: Option<String>,
    other_gender: Option<String>,
}

pub async fn user_top_streaks<'e, E>(
    exec: E,
    user_id: Uuid,
    limit: i32,
) -> AppResult<Vec<TopStreakEntry>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<TopStreakRow> = sqlx::query_as(
        "SELECT ps.user_a_id, ps.user_b_id, ps.current_streak, ps.best_streak, ps.last_streak_date, \
                ps.a_hugged_today, ps.b_hugged_today, ps.today_date, \
                u.username AS other_username, u.display_name AS other_display_name, u.gender AS other_gender \
         FROM pair_streaks ps \
         JOIN users u ON u.id = CASE WHEN ps.user_a_id = $1::uuid THEN ps.user_b_id ELSE ps.user_a_id END \
         WHERE (ps.user_a_id = $1::uuid OR ps.user_b_id = $1::uuid) AND ps.current_streak > 0 \
         ORDER BY ps.current_streak DESC LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(exec)
    .await?;

    let today = Utc::now().date_naive();
    let mut result = Vec::with_capacity(rows.len());
    for r in rows {
        let current =
            evaluate_streak_freshness(r.current_streak, r.last_streak_date, r.a_hugged_today, r.b_hugged_today, r.today_date, today);
        if current <= 0 {
            continue;
        }
        let tier = compute_streak_tier(current);
        let other_id = if r.user_a_id == user_id {
            r.user_b_id
        } else {
            r.user_a_id
        };
        result.push(TopStreakEntry {
            user_id: other_id,
            username: r.other_username,
            display_name: r.other_display_name,
            gender: r.other_gender,
            current_streak: current,
            best_streak: r.best_streak,
            tier_level: tier.level,
            tier_name: tier.name.to_string(),
            tier_key: tier.key.to_string(),
        });
    }
    Ok(result)
}

fn evaluate_streak_freshness(
    current_streak: i32,
    last_streak_date: Option<NaiveDate>,
    a_today: bool,
    b_today: bool,
    today_date: NaiveDate,
    today: NaiveDate,
) -> i32 {
    if today_date == today {
        return current_streak;
    }
    let yesterday = today.pred_opt().unwrap_or(today);
    if a_today && b_today && today_date == yesterday {
        return current_streak;
    }
    if let Some(d) = last_streak_date {
        if d == yesterday || d == today {
            return current_streak;
        }
    }
    0
}

#[derive(sqlx::FromRow)]
struct CalendarRow {
    hug_date: NaiveDate,
    hug_count: i64,
    completed: bool,
}

pub async fn pair_streak_calendar<'e, E>(
    exec: E,
    a: Uuid,
    b: Uuid,
    since: DateTime<Utc>,
) -> AppResult<Vec<StreakCalendarDay>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<CalendarRow> = sqlx::query_as(
        "SELECT (COALESCE(h.accepted_at, h.created_at))::date AS hug_date, \
                COUNT(*)::bigint AS hug_count, \
                (COUNT(*) FILTER (WHERE h.giver_id = $1::uuid) > 0 AND COUNT(*) FILTER (WHERE h.giver_id = $2::uuid) > 0)::bool AS completed \
         FROM hugs h \
         WHERE ((h.giver_id = $1::uuid AND h.receiver_id = $2::uuid) OR (h.giver_id = $2::uuid AND h.receiver_id = $1::uuid)) \
           AND h.status = 'completed' AND COALESCE(h.accepted_at, h.created_at) >= $3 \
         GROUP BY hug_date ORDER BY hug_date",
    )
    .bind(a)
    .bind(b)
    .bind(since)
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| StreakCalendarDay {
            date: r.hug_date,
            hug_count: r.hug_count,
            completed: r.completed,
        })
        .collect())
}

// Helpful constant exports
pub const DEFAULT_COOLDOWN_SECONDS: i32 = 3600;
pub const COOLDOWN_REDUCTION_PER_UPGRADE: i32 = 600;
pub const UPGRADE_COST: i32 = 5;
pub const DECLINE_COOLDOWN_SECONDS: i64 = 300;
pub const STREAK_HISTORY_WINDOW: Duration = Duration::days(90);
