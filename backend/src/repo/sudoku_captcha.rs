use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Debug, sqlx::FromRow)]
pub struct SudokuCaptcha {
    pub id: Uuid,
    pub user_id: Uuid,
    pub puzzle: Value,
    pub solution: Value,
    pub errors: i32,
    pub passed: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CasinoCaptcha {
    pub id: Uuid,
    pub user_id: Uuid,
    pub passed: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub async fn create_sudoku<'e, E>(
    exec: E,
    user_id: Uuid,
    puzzle: Value,
    solution: Value,
    expires_at: DateTime<Utc>,
) -> AppResult<SudokuCaptcha>
where
    E: PgExecutor<'e>,
{
    let row: SudokuCaptcha = sqlx::query_as(
        "INSERT INTO sudoku_captchas (user_id, puzzle, solution, expires_at) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(user_id)
    .bind(puzzle)
    .bind(solution)
    .bind(expires_at)
    .fetch_one(exec)
    .await?;
    Ok(row)
}

pub async fn get_sudoku<'e, E>(exec: E, id: Uuid) -> AppResult<Option<SudokuCaptcha>>
where
    E: PgExecutor<'e>,
{
    let row: Option<SudokuCaptcha> = sqlx::query_as("SELECT * FROM sudoku_captchas WHERE id = $1")
        .bind(id)
        .fetch_optional(exec)
        .await?;
    Ok(row)
}

pub async fn increment_sudoku_errors<'e, E>(exec: E, id: Uuid) -> AppResult<SudokuCaptcha>
where
    E: PgExecutor<'e>,
{
    let row: SudokuCaptcha =
        sqlx::query_as("UPDATE sudoku_captchas SET errors = errors + 1 WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_one(exec)
            .await?;
    Ok(row)
}

pub async fn mark_sudoku_passed<'e, E>(exec: E, id: Uuid) -> AppResult<SudokuCaptcha>
where
    E: PgExecutor<'e>,
{
    let row: SudokuCaptcha =
        sqlx::query_as("UPDATE sudoku_captchas SET passed = TRUE WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_one(exec)
            .await?;
    Ok(row)
}

pub async fn delete_sudoku<'e, E>(exec: E, id: Uuid) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("DELETE FROM sudoku_captchas WHERE id = $1")
        .bind(id)
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn create_casino<'e, E>(
    exec: E,
    user_id: Uuid,
    expires_at: DateTime<Utc>,
) -> AppResult<CasinoCaptcha>
where
    E: PgExecutor<'e>,
{
    let row: CasinoCaptcha = sqlx::query_as(
        "INSERT INTO casino_captchas (user_id, expires_at) VALUES ($1, $2) RETURNING *",
    )
    .bind(user_id)
    .bind(expires_at)
    .fetch_one(exec)
    .await?;
    Ok(row)
}

pub async fn get_casino<'e, E>(exec: E, id: Uuid) -> AppResult<Option<CasinoCaptcha>>
where
    E: PgExecutor<'e>,
{
    let row: Option<CasinoCaptcha> = sqlx::query_as("SELECT * FROM casino_captchas WHERE id = $1")
        .bind(id)
        .fetch_optional(exec)
        .await?;
    Ok(row)
}

pub async fn mark_casino_passed<'e, E>(exec: E, id: Uuid) -> AppResult<CasinoCaptcha>
where
    E: PgExecutor<'e>,
{
    let row: CasinoCaptcha =
        sqlx::query_as("UPDATE casino_captchas SET passed = TRUE WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_one(exec)
            .await?;
    Ok(row)
}

pub async fn delete_casino<'e, E>(exec: E, id: Uuid) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("DELETE FROM casino_captchas WHERE id = $1")
        .bind(id)
        .execute(exec)
        .await?;
    Ok(())
}
