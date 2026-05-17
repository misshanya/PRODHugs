use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::UserNote;

#[derive(sqlx::FromRow)]
struct Row {
    author_id: Uuid,
    target_id: Uuid,
    content: String,
    updated_at: DateTime<Utc>,
}

impl From<Row> for UserNote {
    fn from(r: Row) -> Self {
        UserNote {
            author_id: r.author_id,
            target_id: r.target_id,
            content: r.content,
            updated_at: r.updated_at,
            target_username: String::new(),
            target_display_name: None,
        }
    }
}

pub async fn get<'e, E>(exec: E, author_id: Uuid, target_id: Uuid) -> AppResult<Option<UserNote>>
where
    E: PgExecutor<'e>,
{
    let row: Option<Row> = sqlx::query_as(
        "SELECT author_id, target_id, content, updated_at FROM user_notes \
         WHERE author_id = $1 AND target_id = $2",
    )
    .bind(author_id)
    .bind(target_id)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn upsert<'e, E>(
    exec: E,
    author_id: Uuid,
    target_id: Uuid,
    content: &str,
) -> AppResult<UserNote>
where
    E: PgExecutor<'e>,
{
    let row: Row = sqlx::query_as(
        "INSERT INTO user_notes (author_id, target_id, content, updated_at) \
         VALUES ($1, $2, $3, NOW()) \
         ON CONFLICT (author_id, target_id) DO UPDATE \
         SET content = EXCLUDED.content, updated_at = NOW() \
         RETURNING author_id, target_id, content, updated_at",
    )
    .bind(author_id)
    .bind(target_id)
    .bind(content)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn delete<'e, E>(exec: E, author_id: Uuid, target_id: Uuid) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("DELETE FROM user_notes WHERE author_id = $1 AND target_id = $2")
        .bind(author_id)
        .bind(target_id)
        .execute(exec)
        .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct ListRow {
    author_id: Uuid,
    target_id: Uuid,
    content: String,
    updated_at: DateTime<Utc>,
    target_username: String,
    target_display_name: Option<String>,
}

pub async fn list<'e, E>(
    exec: E,
    author_id: Uuid,
    limit: i32,
    offset: i32,
) -> AppResult<Vec<UserNote>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<ListRow> = sqlx::query_as(
        "SELECT n.author_id, n.target_id, n.content, n.updated_at, \
                u.username AS target_username, u.display_name AS target_display_name \
         FROM user_notes n JOIN users u ON u.id = n.target_id \
         WHERE n.author_id = $1 ORDER BY n.updated_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(author_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| UserNote {
            author_id: r.author_id,
            target_id: r.target_id,
            content: r.content,
            updated_at: r.updated_at,
            target_username: r.target_username,
            target_display_name: r.target_display_name,
        })
        .collect())
}
