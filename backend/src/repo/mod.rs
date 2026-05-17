//! Data-access layer.
//!
//! Each submodule exposes free async functions that accept any
//! `sqlx::PgExecutor`, so callers can pass either `&PgPool` for autocommit
//! work or `&mut *tx` when participating in a service-level transaction.

pub mod announcement;
pub mod balance;
pub mod block;
pub mod daily_reward;
pub mod hug;
pub mod intimacy;
pub mod note;
pub mod sudoku_captcha;
pub mod token;
pub mod user;

use crate::error::AppError;

pub fn is_unique_violation(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => db_err.code().as_deref() == Some("23505"),
        _ => false,
    }
}

pub(crate) fn map_unique(err: sqlx::Error, on_conflict: AppError) -> AppError {
    if is_unique_violation(&err) {
        on_conflict
    } else {
        AppError::Db(err)
    }
}
