use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::Announcement;

#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    message: String,
    created_at: DateTime<Utc>,
    created_by: Uuid,
    active: bool,
}

impl From<Row> for Announcement {
    fn from(r: Row) -> Self {
        Announcement {
            id: r.id,
            message: r.message,
            created_at: r.created_at,
            created_by: r.created_by,
            active: r.active,
        }
    }
}

pub async fn get_active<'e, E>(exec: E) -> AppResult<Option<Announcement>>
where
    E: PgExecutor<'e>,
{
    let row: Option<Row> = sqlx::query_as(
        "SELECT id, message, created_at, created_by, active FROM announcements \
         WHERE active = TRUE ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn get_active_for_user<'e, E>(exec: E, user_id: Uuid) -> AppResult<Option<Announcement>>
where
    E: PgExecutor<'e>,
{
    let row: Option<(Uuid, String, DateTime<Utc>, Uuid)> = sqlx::query_as(
        "SELECT a.id, a.message, a.created_at, a.created_by FROM announcements a \
         WHERE a.active = TRUE AND NOT EXISTS ( \
             SELECT 1 FROM announcement_dismissals ad WHERE ad.announcement_id = a.id AND ad.user_id = $1 \
         ) \
         ORDER BY a.created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(|(id, message, created_at, created_by)| Announcement {
        id,
        message,
        created_at,
        created_by,
        active: true,
    }))
}

pub async fn create<'e, E>(exec: E, message: &str, created_by: Uuid) -> AppResult<Announcement>
where
    E: PgExecutor<'e>,
{
    let row: Row = sqlx::query_as(
        "WITH deactivated AS (UPDATE announcements SET active = FALSE WHERE active = TRUE) \
         INSERT INTO announcements (message, created_by) VALUES ($1, $2) \
         RETURNING id, message, created_at, created_by, active",
    )
    .bind(message)
    .bind(created_by)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn deactivate<'e, E>(exec: E, id: Uuid) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("UPDATE announcements SET active = FALSE WHERE id = $1")
        .bind(id)
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn dismiss<'e, E>(exec: E, announcement_id: Uuid, user_id: Uuid) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO announcement_dismissals (announcement_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(announcement_id)
    .bind(user_id)
    .execute(exec)
    .await?;
    Ok(())
}
