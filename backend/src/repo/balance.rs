use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::Balance;

#[derive(sqlx::FromRow)]
struct BalanceRow {
    user_id: Uuid,
    amount: i32,
    updated_at: DateTime<Utc>,
}

impl From<BalanceRow> for Balance {
    fn from(r: BalanceRow) -> Self {
        Balance {
            user_id: r.user_id,
            amount: r.amount,
            updated_at: r.updated_at,
        }
    }
}

pub async fn get<'e, E>(exec: E, user_id: Uuid) -> AppResult<Balance>
where
    E: PgExecutor<'e>,
{
    let row: Option<BalanceRow> =
        sqlx::query_as("SELECT user_id, amount, updated_at FROM balances WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(exec)
            .await?;
    Ok(row.map(Into::into).unwrap_or(Balance {
        user_id,
        amount: 0,
        updated_at: Utc::now(),
    }))
}

pub async fn ensure<'e, E>(exec: E, user_id: Uuid) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("INSERT INTO balances (user_id, amount) VALUES ($1, 0) ON CONFLICT (user_id) DO NOTHING")
        .bind(user_id)
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn add<'e, E>(exec: E, user_id: Uuid, delta: i32) -> AppResult<Balance>
where
    E: PgExecutor<'e>,
{
    let row: BalanceRow = sqlx::query_as(
        "INSERT INTO balances (user_id, amount, updated_at) VALUES ($1, $2, now()) \
         ON CONFLICT (user_id) DO UPDATE SET amount = balances.amount + $2, updated_at = now() \
         RETURNING user_id, amount, updated_at",
    )
    .bind(user_id)
    .bind(delta)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn deduct<'e, E>(exec: E, user_id: Uuid, delta: i32) -> AppResult<Option<Balance>>
where
    E: PgExecutor<'e>,
{
    let row: Option<BalanceRow> = sqlx::query_as(
        "UPDATE balances SET amount = amount - $2, updated_at = now() \
         WHERE user_id = $1 AND amount >= $2 RETURNING user_id, amount, updated_at",
    )
    .bind(user_id)
    .bind(delta)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(Into::into))
}

pub async fn admin_set<'e, E>(exec: E, user_id: Uuid, amount: i32) -> AppResult<Balance>
where
    E: PgExecutor<'e>,
{
    let row: BalanceRow = sqlx::query_as(
        "INSERT INTO balances (user_id, amount, updated_at) VALUES ($1, $2, now()) \
         ON CONFLICT (user_id) DO UPDATE SET amount = $2, updated_at = now() \
         RETURNING user_id, amount, updated_at",
    )
    .bind(user_id)
    .bind(amount)
    .fetch_one(exec)
    .await
    .map_err(AppError::Db)?;
    Ok(row.into())
}
