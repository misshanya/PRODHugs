use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{User, UserNote, MAX_USER_NOTE_LENGTH};
use crate::repo;

pub struct NoteService {
    pool: PgPool,
}

impl NoteService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Accepts a UUID, "@username", or bare username; returns the matching user.
    pub async fn resolve_target(&self, raw: &str) -> AppResult<User> {
        if let Ok(id) = Uuid::parse_str(raw) {
            return repo::user::get_by_id(&self.pool, id).await;
        }
        let username = raw.strip_prefix('@').unwrap_or(raw);
        if username.is_empty() {
            return Err(AppError::UserNotFound);
        }
        repo::user::get_by_username(&self.pool, username).await
    }

    pub async fn get(&self, author_id: Uuid, target_id: Uuid) -> AppResult<Option<UserNote>> {
        repo::note::get(&self.pool, author_id, target_id).await
    }

    pub async fn upsert(
        &self,
        author_id: Uuid,
        target_id: Uuid,
        raw_content: &str,
    ) -> AppResult<UserNote> {
        let content = raw_content.trim();
        if content.is_empty() {
            return Err(AppError::NoteInvalid);
        }
        let len = UnicodeSegmentation::graphemes(content, true).count();
        // Match the Go implementation which uses `utf8.RuneCountInString` —
        // i.e. counts Unicode code points. Use chars() for parity.
        let codepoints = content.chars().count();
        let _ = len; // graphemes preserved for parity checks during local dev
        if codepoints > MAX_USER_NOTE_LENGTH {
            return Err(AppError::NoteInvalid);
        }
        repo::note::upsert(&self.pool, author_id, target_id, content).await
    }

    pub async fn delete(&self, author_id: Uuid, target_id: Uuid) -> AppResult<()> {
        repo::note::delete(&self.pool, author_id, target_id).await
    }

    pub async fn list(
        &self,
        author_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> AppResult<Vec<UserNote>> {
        repo::note::list(&self.pool, author_id, limit, offset).await
    }
}
