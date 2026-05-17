//! Database adapter helpers.
//!
//! The original Go transactor wrapped a `pgxpool.Pool` and forwarded
//! transaction handles via `context.Value`. In Rust we lean on the natural
//! sqlx pattern: repositories accept `impl sqlx::PgExecutor`, and the service
//! layer either passes `&pool` or `&mut *tx` depending on whether a query
//! needs to be transactional.
//!
//! `Transactor` exists as a thin owner of the `PgPool` plus a helper for the
//! common `pool.begin() -> commit/rollback` pattern, so callers don't repeat
//! the same boilerplate at every transactional service entry point.

use std::future::Future;

use sqlx::{PgPool, Postgres, Transaction};

use crate::error::AppError;

pub struct Transactor {
    pool: PgPool,
}

impl Transactor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn begin(&self) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
        self.pool.begin().await
    }

    /// Run `f` inside a transaction. Commits if `f` returns `Ok`, rolls back
    /// on `Err`. Mirrors the Go `Transactor.RunInTx` contract.
    pub async fn run_in_tx<F, Fut, T>(&self, f: F) -> Result<T, AppError>
    where
        F: for<'c> FnOnce(&'c mut Transaction<'static, Postgres>) -> Fut,
        Fut: Future<Output = Result<T, AppError>>,
    {
        let mut tx = self.pool.begin().await?;
        match f(&mut tx).await {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(err) => {
                let _ = tx.rollback().await;
                Err(err)
            }
        }
    }
}
