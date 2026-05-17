use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::BlockedUser;

pub async fn block<'e, E>(exec: E, blocker: Uuid, blocked: Uuid) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("INSERT INTO user_blocks (blocker_id, blocked_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(blocker)
        .bind(blocked)
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn unblock<'e, E>(exec: E, blocker: Uuid, blocked: Uuid) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("DELETE FROM user_blocks WHERE blocker_id = $1 AND blocked_id = $2")
        .bind(blocker)
        .bind(blocked)
        .execute(exec)
        .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    username: String,
    gender: Option<String>,
    display_name: Option<String>,
    tag: Option<String>,
    special_tag: Option<String>,
    created_at: DateTime<Utc>,
}

pub async fn list_blocked<'e, E>(exec: E, blocker: Uuid) -> AppResult<Vec<BlockedUser>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT u.id, u.username, u.gender, u.display_name, u.tag, u.special_tag, ub.created_at \
         FROM user_blocks ub JOIN users u ON u.id = ub.blocked_id \
         WHERE ub.blocker_id = $1 ORDER BY ub.created_at DESC",
    )
    .bind(blocker)
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| BlockedUser {
            id: r.id,
            username: r.username,
            gender: r.gender,
            display_name: r.display_name,
            tag: r.tag,
            special_tag: r.special_tag,
            created_at: r.created_at,
        })
        .collect())
}

pub async fn is_blocked_by_either<'e, E>(exec: E, a: Uuid, b: Uuid) -> AppResult<bool>
where
    E: PgExecutor<'e>,
{
    let (blocked,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM user_blocks WHERE (blocker_id = $1 AND blocked_id = $2) OR (blocker_id = $2 AND blocked_id = $1))",
    )
    .bind(a)
    .bind(b)
    .fetch_one(exec)
    .await?;
    Ok(blocked)
}
