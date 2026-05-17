use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{compute_intimacy_info, compute_tier, ConnectionItem, LeaderboardPairEntry, PairIntimacy};

#[derive(sqlx::FromRow)]
struct PairRow {
    user_a_id: Uuid,
    user_b_id: Uuid,
    raw_score: i32,
    last_hug_at: DateTime<Utc>,
    last_decay_at: DateTime<Utc>,
}

impl From<PairRow> for PairIntimacy {
    fn from(r: PairRow) -> Self {
        PairIntimacy {
            user_a_id: r.user_a_id,
            user_b_id: r.user_b_id,
            raw_score: r.raw_score,
            last_hug_at: r.last_hug_at,
            last_decay_at: r.last_decay_at,
        }
    }
}

pub async fn get_pair<'e, E>(exec: E, a: Uuid, b: Uuid) -> AppResult<Option<PairIntimacy>>
where
    E: PgExecutor<'e>,
{
    let row: Option<PairRow> = sqlx::query_as(
        "SELECT user_a_id, user_b_id, raw_score, last_hug_at, last_decay_at \
         FROM pair_intimacy WHERE user_a_id = LEAST($1::uuid, $2::uuid) AND user_b_id = GREATEST($1::uuid, $2::uuid)",
    )
    .bind(a)
    .bind(b)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn upsert_pair<'e, E>(exec: E, a: Uuid, b: Uuid) -> AppResult<PairIntimacy>
where
    E: PgExecutor<'e>,
{
    let row: PairRow = sqlx::query_as(
        "INSERT INTO pair_intimacy (user_a_id, user_b_id, raw_score, last_hug_at, last_decay_at) \
         VALUES (LEAST($1::uuid, $2::uuid), GREATEST($1::uuid, $2::uuid), 1, NOW(), NOW()) \
         ON CONFLICT (user_a_id, user_b_id) DO UPDATE SET raw_score = pair_intimacy.raw_score + 1, last_hug_at = NOW() \
         RETURNING user_a_id, user_b_id, raw_score, last_hug_at, last_decay_at",
    )
    .bind(a)
    .bind(b)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn apply_decay<'e, E>(exec: E) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "UPDATE pair_intimacy \
         SET raw_score = GREATEST(raw_score - FLOOR(EXTRACT(EPOCH FROM (NOW() - last_decay_at)) / 259200)::int, 0), \
             last_decay_at = NOW() \
         WHERE last_decay_at < NOW() - INTERVAL '3 days' AND raw_score > 0",
    )
    .execute(exec)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct ConnectionRow {
    user_a_id: Uuid,
    user_b_id: Uuid,
    raw_score: i32,
    last_hug_at: DateTime<Utc>,
    last_decay_at: DateTime<Utc>,
    username: String,
    gender: Option<String>,
    display_name: Option<String>,
}

pub async fn user_connections<'e, E>(
    exec: E,
    user_id: Uuid,
    limit: i32,
    offset: i32,
) -> AppResult<Vec<ConnectionItem>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<ConnectionRow> = sqlx::query_as(
        "SELECT pi.user_a_id, pi.user_b_id, pi.raw_score, pi.last_hug_at, pi.last_decay_at, \
                u.username, u.gender, u.display_name \
         FROM pair_intimacy pi JOIN users u ON u.id = CASE WHEN pi.user_a_id = $1::uuid THEN pi.user_b_id ELSE pi.user_a_id END \
         WHERE (pi.user_a_id = $1::uuid OR pi.user_b_id = $1::uuid) AND pi.raw_score > 0 \
         ORDER BY pi.raw_score DESC LIMIT $2 OFFSET $3",
    )
    .bind(user_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let other = if r.user_b_id == user_id {
                r.user_a_id
            } else {
                r.user_b_id
            };
            ConnectionItem {
                user_id: other,
                username: r.username,
                gender: r.gender,
                display_name: r.display_name,
                intimacy: compute_intimacy_info(r.raw_score),
            }
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct LbRow {
    user_a_id: Uuid,
    user_b_id: Uuid,
    raw_score: i32,
    user_a_username: String,
    user_a_display_name: Option<String>,
    user_b_username: String,
    user_b_display_name: Option<String>,
}

pub async fn leaderboard<'e, E>(
    exec: E,
    limit: i32,
    offset: i32,
) -> AppResult<Vec<LeaderboardPairEntry>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<LbRow> = sqlx::query_as(
        "SELECT pi.user_a_id, pi.user_b_id, pi.raw_score, \
                ua.username AS user_a_username, ua.display_name AS user_a_display_name, \
                ub.username AS user_b_username, ub.display_name AS user_b_display_name \
         FROM pair_intimacy pi JOIN users ua ON ua.id = pi.user_a_id JOIN users ub ON ub.id = pi.user_b_id \
         WHERE pi.raw_score > 0 ORDER BY pi.raw_score DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let tier = compute_tier(r.raw_score);
            LeaderboardPairEntry {
                user_a_id: r.user_a_id,
                user_a_username: r.user_a_username,
                user_a_display_name: r.user_a_display_name,
                user_b_id: r.user_b_id,
                user_b_username: r.user_b_username,
                user_b_display_name: r.user_b_display_name,
                raw_score: r.raw_score,
                tier: tier.level,
                tier_name: tier.name.to_string(),
            }
        })
        .collect())
}
