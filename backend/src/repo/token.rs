use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::AppResult;

pub async fn save<'e, E>(exec: E, jti: &str, user_id: Uuid, expires_at: i64) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO refresh_tokens (jti, user_id, expires_at) VALUES ($1, $2::uuid, to_timestamp($3))",
    )
    .bind(jti)
    .bind(user_id)
    .bind(expires_at)
    .execute(exec)
    .await?;
    Ok(())
}

pub async fn is_active<'e, E>(exec: E, jti: &str) -> AppResult<bool>
where
    E: PgExecutor<'e>,
{
    let (active,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM refresh_tokens WHERE jti = $1 AND revoked_at IS NULL AND expires_at > NOW())",
    )
    .bind(jti)
    .fetch_one(exec)
    .await?;
    Ok(active)
}

pub async fn revoke<'e, E>(exec: E, jti: &str) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE jti = $1 AND revoked_at IS NULL")
        .bind(jti)
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn revoke_all_for_user<'e, E>(exec: E, user_id: Uuid) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1::uuid AND revoked_at IS NULL",
    )
    .bind(user_id)
    .execute(exec)
    .await?;
    Ok(())
}
