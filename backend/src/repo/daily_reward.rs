use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::DailyReward;

#[derive(sqlx::FromRow)]
struct Row {
    user_id: Uuid,
    last_claimed_at: DateTime<Utc>,
    streak_days: i32,
}

impl From<Row> for DailyReward {
    fn from(r: Row) -> Self {
        DailyReward {
            user_id: r.user_id,
            last_claimed_at: r.last_claimed_at,
            streak_days: r.streak_days,
        }
    }
}

pub async fn get<'e, E>(exec: E, user_id: Uuid) -> AppResult<Option<DailyReward>>
where
    E: PgExecutor<'e>,
{
    let row: Option<Row> = sqlx::query_as(
        "SELECT user_id, last_claimed_at, streak_days FROM daily_rewards WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn claim<'e, E>(exec: E, user_id: Uuid) -> AppResult<DailyReward>
where
    E: PgExecutor<'e>,
{
    let row: Row = sqlx::query_as(
        "INSERT INTO daily_rewards (user_id, last_claimed_at, streak_days) \
         VALUES ($1, now(), 1) \
         ON CONFLICT (user_id) DO UPDATE SET \
             streak_days = CASE \
                 WHEN daily_rewards.last_claimed_at::date = (now() - interval '1 day')::date THEN daily_rewards.streak_days + 1 \
                 WHEN daily_rewards.last_claimed_at::date = now()::date THEN daily_rewards.streak_days \
                 ELSE 1 \
             END, \
             last_claimed_at = CASE \
                 WHEN daily_rewards.last_claimed_at::date = now()::date THEN daily_rewards.last_claimed_at \
                 ELSE now() \
             END \
         RETURNING user_id, last_claimed_at, streak_days",
    )
    .bind(user_id)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}
