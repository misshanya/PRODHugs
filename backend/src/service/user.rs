//! User-related business logic.
//!
//! Mirrors the Go `internal/service/user` package: registration, login, refresh
//! tokens, profile updates, captcha (sudoku + casino), announcements, Telegram
//! linking, and admin operations (ban/unban, promotion, balance edits, …).

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto;
use crate::error::{AppError, AppResult};
use crate::jwt::JwtManager;
use crate::models::{
    AdminStats, AdminUser, Announcement, Balance, CreateUser, User,
};
use crate::repo;
use crate::telegram::{LinkStore, LoginStore, TelegramUserInfo};

pub type AnnouncementCallback = Arc<dyn Fn(&Announcement) + Send + Sync>;
pub type AnnouncementRemovedCallback = Arc<dyn Fn(Uuid) + Send + Sync>;
pub type PromotionUpdatedCallback = Arc<dyn Fn() + Send + Sync>;

pub struct UserService {
    pool: PgPool,
    jwt: Arc<JwtManager>,
    link_store: Arc<LinkStore>,
    login_store: Arc<LoginStore>,
    bot_username: String,

    on_announcement_created: RwLock<Option<AnnouncementCallback>>,
    on_announcement_removed: RwLock<Option<AnnouncementRemovedCallback>>,
    on_promotion_updated: RwLock<Option<PromotionUpdatedCallback>>,
}

#[derive(Debug, Clone)]
pub struct CaptchaResult {
    pub correct: bool,
    pub errors: i32,
    pub failed: bool,
}

#[derive(Debug, Clone)]
pub struct CasinoSpinResult {
    pub win: bool,
    pub captcha_token: Option<String>,
    pub cooldown_until: Option<DateTime<Utc>>,
}

const TAG_CHANGE_COST: i32 = 5;
const PROMOTION_FAR_FUTURE_HOURS: i64 = 100 * 365 * 24;
const VIP_RESET_BUDGET_SECONDS: i32 = 86_400;
const VIP_COOLDOWN_HOURS: i64 = 6;

impl UserService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool: PgPool,
        jwt: Arc<JwtManager>,
        link_store: Arc<LinkStore>,
        login_store: Arc<LoginStore>,
        bot_username: String,
    ) -> Self {
        Self {
            pool,
            jwt,
            link_store,
            login_store,
            bot_username,
            on_announcement_created: RwLock::new(None),
            on_announcement_removed: RwLock::new(None),
            on_promotion_updated: RwLock::new(None),
        }
    }

    pub fn set_on_announcement_created(&self, cb: AnnouncementCallback) {
        *self.on_announcement_created.write() = Some(cb);
    }
    pub fn set_on_announcement_removed(&self, cb: AnnouncementRemovedCallback) {
        *self.on_announcement_removed.write() = Some(cb);
    }
    pub fn set_on_promotion_updated(&self, cb: PromotionUpdatedCallback) {
        *self.on_promotion_updated.write() = Some(cb);
    }

    // ── identity ───────────────────────────────────────────────────────

    pub async fn create(
        &self,
        input: CreateUser,
    ) -> AppResult<(User, String, String)> {
        let hash = crypto::generate_hash(&input.password).map_err(AppError::Other)?;
        let mut to_insert = input;
        to_insert.hashed_password = hash;

        let user = repo::user::create(&self.pool, &to_insert).await?;

        let (access, _) = self
            .jwt
            .generate_access_token(user.id, &user.role)
            .map_err(AppError::Other)?;
        let (refresh, jti, exp) = self
            .jwt
            .generate_refresh_token(user.id)
            .map_err(AppError::Other)?;
        repo::token::save(&self.pool, &jti, user.id, exp).await?;

        Ok((user, access, refresh))
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> AppResult<(User, String, String)> {
        let user = repo::user::get_by_username(&self.pool, username).await?;
        let ok = crypto::compare_password_and_hash(password, &user.hashed_password)
            .map_err(AppError::Other)?;
        if !ok {
            return Err(AppError::InvalidCredentials);
        }
        if user.banned_at.is_some() {
            return Err(AppError::UserBanned);
        }

        let (access, _) = self
            .jwt
            .generate_access_token(user.id, &user.role)
            .map_err(AppError::Other)?;
        let (refresh, jti, exp) = self
            .jwt
            .generate_refresh_token(user.id)
            .map_err(AppError::Other)?;
        repo::token::save(&self.pool, &jti, user.id, exp).await?;

        Ok((user, access, refresh))
    }

    pub async fn get_by_id(&self, id: Uuid) -> AppResult<User> {
        repo::user::get_by_id(&self.pool, id).await
    }

    pub async fn get_by_username(&self, username: &str) -> AppResult<User> {
        repo::user::get_by_username(&self.pool, username).await
    }

    pub async fn list_vip(&self) -> AppResult<Vec<User>> {
        repo::user::list_vip(&self.pool).await
    }

    pub async fn update_settings(
        &self,
        id: Uuid,
        gender: Option<&str>,
        display_name: Option<&str>,
        tag: Option<&str>,
    ) -> AppResult<User> {
        // Setting a non-empty tag that differs from the current one costs coins.
        if let Some(new_tag) = tag {
            if !new_tag.is_empty() {
                let current = repo::user::get_by_id(&self.pool, id).await?;
                if current.tag.as_deref() != Some(new_tag) {
                    let mut tx = self.pool.begin().await?;
                    let bal = repo::balance::deduct(&mut *tx, id, TAG_CHANGE_COST).await?;
                    if bal.is_none() {
                        let _ = tx.rollback().await;
                        return Err(AppError::InsufficientBalance);
                    }
                    let result =
                        repo::user::update_settings(&mut *tx, id, gender, display_name, tag)
                            .await?;
                    tx.commit().await?;
                    return Ok(result);
                }
            }
        }
        repo::user::update_settings(&self.pool, id, gender, display_name, tag).await
    }

    pub async fn change_password(
        &self,
        id: Uuid,
        old_password: &str,
        new_password: &str,
    ) -> AppResult<()> {
        let u = repo::user::get_by_id(&self.pool, id).await?;
        let ok = crypto::compare_password_and_hash(old_password, &u.hashed_password)
            .map_err(AppError::Other)?;
        if !ok {
            return Err(AppError::WrongPassword);
        }
        let hash = crypto::generate_hash(new_password).map_err(AppError::Other)?;
        repo::user::update_password(&self.pool, id, &hash).await
    }

    // ── refresh tokens ─────────────────────────────────────────────────

    pub async fn save_refresh_token(
        &self,
        jti: &str,
        user_id: Uuid,
        expires_at_unix: i64,
    ) -> AppResult<()> {
        repo::token::save(&self.pool, jti, user_id, expires_at_unix).await
    }

    pub async fn is_refresh_token_active(&self, jti: &str) -> AppResult<bool> {
        repo::token::is_active(&self.pool, jti).await
    }

    pub async fn revoke_refresh_token(&self, jti: &str) -> AppResult<()> {
        repo::token::revoke(&self.pool, jti).await
    }

    pub async fn revoke_all_user_refresh_tokens(&self, user_id: Uuid) -> AppResult<()> {
        repo::token::revoke_all_for_user(&self.pool, user_id).await
    }

    // ── Telegram link/unlink ───────────────────────────────────────────

    pub fn generate_link_token(&self, user_id: Uuid) -> AppResult<(String, String)> {
        if self.bot_username.is_empty() {
            return Err(AppError::Other(anyhow::anyhow!(
                "telegram linking not configured"
            )));
        }
        let token = self.link_store.generate_token(user_id);
        let url = format!("https://t.me/{}?start={}", self.bot_username, token);
        Ok((token, url))
    }

    pub async fn get_telegram_id(&self, user_id: Uuid) -> AppResult<Option<i64>> {
        repo::user::get_telegram_id(&self.pool, user_id).await
    }

    pub async fn unlink_telegram(&self, user_id: Uuid) -> AppResult<User> {
        repo::user::clear_telegram_id(&self.pool, user_id).await
    }

    // ── promotions ─────────────────────────────────────────────────────

    pub async fn promote_user(
        &self,
        id: Uuid,
        bid: i32,
        message: Option<&str>,
    ) -> AppResult<User> {
        let u = repo::user::get_by_id(&self.pool, id).await?;
        let now = Utc::now();
        let mut tx = self.pool.begin().await?;
        let is_promoted = u.promoted_until.map(|t| t > now).unwrap_or(false);

        let (cost, promoted_until) = if is_promoted {
            let diff = bid - u.promotion_bid;
            let cost = if diff < 0 { 0 } else { diff };
            (cost, u.promoted_until.unwrap_or(now))
        } else {
            (bid, now + chrono::Duration::hours(PROMOTION_FAR_FUTURE_HOURS))
        };

        if cost > 0 {
            let bal = repo::balance::deduct(&mut *tx, id, cost).await?;
            if bal.is_none() {
                let _ = tx.rollback().await;
                return Err(AppError::InsufficientBalance);
            }
        }

        let result = repo::user::promote(&mut *tx, id, promoted_until, message, bid).await?;
        tx.commit().await?;

        if let Some(cb) = self.on_promotion_updated.read().clone() {
            cb();
        }
        Ok(result)
    }

    pub async fn clear_expired_promotions(&self) -> AppResult<i64> {
        let vips = repo::user::list_vip(&self.pool).await?;
        let mut expired = 0i64;
        for (i, u) in vips.iter().enumerate() {
            if i < 3 {
                let new_remaining = u.vip_remaining_seconds - 60;
                if new_remaining <= 0 {
                    if repo::user::admin_clear_promotion(&self.pool, u.id)
                        .await
                        .is_ok()
                    {
                        let _ = repo::user::set_vip_cooldown(
                            &self.pool,
                            u.id,
                            Utc::now() + chrono::Duration::hours(VIP_COOLDOWN_HOURS),
                            VIP_RESET_BUDGET_SECONDS,
                        )
                        .await;
                        expired += 1;
                    }
                } else {
                    let _ = repo::user::update_vip_budget(&self.pool, u.id, new_remaining).await;
                }
            }
        }
        if expired > 0 {
            if let Some(cb) = self.on_promotion_updated.read().clone() {
                cb();
            }
        }
        Ok(expired)
    }

    // ── admin operations ───────────────────────────────────────────────

    pub async fn admin_stats(&self) -> AppResult<AdminStats> {
        repo::user::admin_stats(&self.pool).await
    }

    pub async fn list_admin(&self, limit: i32, offset: i32) -> AppResult<Vec<AdminUser>> {
        repo::user::list_admin(&self.pool, limit, offset).await
    }

    pub async fn search_admin(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> AppResult<Vec<AdminUser>> {
        repo::user::search_admin(&self.pool, query, limit, offset).await
    }

    pub async fn ban(&self, id: Uuid) -> AppResult<User> {
        repo::user::ban(&self.pool, id).await
    }
    pub async fn unban(&self, id: Uuid) -> AppResult<User> {
        repo::user::unban(&self.pool, id).await
    }
    pub async fn admin_update_username(&self, id: Uuid, username: &str) -> AppResult<User> {
        repo::user::admin_update_username(&self.pool, id, username).await
    }
    pub async fn admin_update_gender(&self, id: Uuid, gender: Option<&str>) -> AppResult<User> {
        repo::user::admin_update_gender(&self.pool, id, gender).await
    }
    pub async fn admin_update_password(&self, id: Uuid, new_password: &str) -> AppResult<()> {
        let hash = crypto::generate_hash(new_password).map_err(AppError::Other)?;
        repo::user::admin_update_password(&self.pool, id, &hash).await
    }
    pub async fn admin_update_display_name(
        &self,
        id: Uuid,
        display_name: Option<&str>,
    ) -> AppResult<User> {
        repo::user::admin_update_display_name(&self.pool, id, display_name).await
    }
    pub async fn admin_update_tag(&self, id: Uuid, tag: Option<&str>) -> AppResult<User> {
        repo::user::admin_update_tag(&self.pool, id, tag).await
    }
    pub async fn admin_update_special_tag(
        &self,
        id: Uuid,
        special_tag: Option<&str>,
    ) -> AppResult<User> {
        repo::user::admin_update_special_tag(&self.pool, id, special_tag).await
    }
    pub async fn admin_update_captcha_type(
        &self,
        id: Uuid,
        captcha_type: &str,
    ) -> AppResult<User> {
        repo::user::admin_update_captcha_type(&self.pool, id, captcha_type).await
    }
    pub async fn admin_delete(&self, id: Uuid) -> AppResult<()> {
        repo::user::admin_delete(&self.pool, id).await
    }
    pub async fn admin_clear_promotion(&self, id: Uuid) -> AppResult<User> {
        let user = repo::user::admin_clear_promotion(&self.pool, id).await?;
        if let Some(cb) = self.on_promotion_updated.read().clone() {
            cb();
        }
        Ok(user)
    }
    pub async fn admin_update_balance(&self, id: Uuid, amount: i32) -> AppResult<Balance> {
        repo::balance::admin_set(&self.pool, id, amount).await
    }

    // ── announcements ──────────────────────────────────────────────────

    pub async fn active_announcement(&self, user_id: Uuid) -> AppResult<Option<Announcement>> {
        repo::announcement::get_active_for_user(&self.pool, user_id).await
    }

    pub async fn create_announcement(
        &self,
        admin_id: Uuid,
        message: &str,
    ) -> AppResult<Announcement> {
        let a = repo::announcement::create(&self.pool, message, admin_id).await?;
        if let Some(cb) = self.on_announcement_created.read().clone() {
            cb(&a);
        }
        Ok(a)
    }

    pub async fn deactivate_announcement(&self, id: Uuid) -> AppResult<()> {
        repo::announcement::deactivate(&self.pool, id).await?;
        if let Some(cb) = self.on_announcement_removed.read().clone() {
            cb(id);
        }
        Ok(())
    }

    pub async fn dismiss_announcement(&self, user_id: Uuid, announcement_id: Uuid) -> AppResult<()> {
        repo::announcement::dismiss(&self.pool, announcement_id, user_id).await
    }

    // ── captcha (sudoku) ───────────────────────────────────────────────

    pub async fn generate_sudoku_captcha(
        &self,
        user_id: Uuid,
    ) -> AppResult<(Uuid, Vec<Vec<i32>>)> {
        let (puzzle, solution) = crate::sudoku::generate();
        let puzzle_json = sudoku_to_json(&puzzle);
        let solution_json = sudoku_to_json(&solution);
        let expires = Utc::now() + chrono::Duration::minutes(10);
        let captcha = repo::sudoku_captcha::create_sudoku(
            &self.pool,
            user_id,
            puzzle_json,
            solution_json,
            expires,
        )
        .await?;
        let mut grid = Vec::with_capacity(9);
        for row in &puzzle {
            grid.push(row.iter().map(|&v| v as i32).collect());
        }
        Ok((captcha.id, grid))
    }

    pub async fn verify_sudoku_cell(
        &self,
        captcha_id: Uuid,
        user_id: Uuid,
        row: i32,
        col: i32,
        value: i32,
    ) -> AppResult<CaptchaResult> {
        let captcha = repo::sudoku_captcha::get_sudoku(&self.pool, captcha_id)
            .await?
            .ok_or(AppError::CaptchaNotFound)?;
        if captcha.user_id != user_id {
            return Err(AppError::CaptchaForbidden);
        }
        if captcha.passed {
            return Err(AppError::CaptchaGone);
        }
        let solution: [[i32; 9]; 9] = serde_json::from_value(captcha.solution).unwrap_or([[0; 9]; 9]);
        let (r, c) = (row as usize, col as usize);
        if r >= 9 || c >= 9 {
            return Err(AppError::CaptchaForbidden);
        }
        if solution[r][c] == value {
            return Ok(CaptchaResult {
                correct: true,
                errors: captcha.errors,
                failed: false,
            });
        }
        let updated = repo::sudoku_captcha::increment_sudoku_errors(&self.pool, captcha_id).await?;
        if updated.errors > 3 {
            let cooldown = Utc::now() + chrono::Duration::minutes(10);
            let _ = repo::user::set_captcha_cooldown(&self.pool, user_id, cooldown).await;
            let _ = repo::sudoku_captcha::delete_sudoku(&self.pool, captcha_id).await;
            return Ok(CaptchaResult {
                correct: false,
                errors: updated.errors,
                failed: true,
            });
        }
        Ok(CaptchaResult {
            correct: false,
            errors: updated.errors,
            failed: false,
        })
    }

    pub async fn complete_sudoku(
        &self,
        captcha_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<String> {
        let captcha = repo::sudoku_captcha::get_sudoku(&self.pool, captcha_id)
            .await?
            .ok_or(AppError::CaptchaNotFound)?;
        if captcha.user_id != user_id {
            return Err(AppError::CaptchaForbidden);
        }
        if captcha.errors > 3 {
            return Err(AppError::CaptchaForbidden);
        }
        let _ = repo::sudoku_captcha::mark_sudoku_passed(&self.pool, captcha_id).await?;
        let token = self.jwt.generate_captcha_token(user_id).map_err(AppError::Other)?;
        Ok(token)
    }

    pub async fn generate_casino_captcha(
        &self,
        user_id: Uuid,
    ) -> AppResult<(Uuid, DateTime<Utc>)> {
        let expires = Utc::now() + chrono::Duration::minutes(10);
        let captcha = repo::sudoku_captcha::create_casino(&self.pool, user_id, expires).await?;
        Ok((captcha.id, captcha.expires_at))
    }

    pub async fn spin_casino(
        &self,
        captcha_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<CasinoSpinResult> {
        let captcha = repo::sudoku_captcha::get_casino(&self.pool, captcha_id)
            .await?
            .ok_or(AppError::CaptchaNotFound)?;
        if captcha.user_id != user_id {
            return Err(AppError::CaptchaForbidden);
        }
        if captcha.passed || captcha.expires_at < Utc::now() {
            return Err(AppError::CaptchaGone);
        }

        let win = rand::thread_rng().gen_range(0..4) == 0;
        if win {
            let _ = repo::sudoku_captcha::mark_casino_passed(&self.pool, captcha_id).await?;
            let token = self
                .jwt
                .generate_captcha_token(user_id)
                .map_err(AppError::Other)?;
            return Ok(CasinoSpinResult {
                win: true,
                captcha_token: Some(token),
                cooldown_until: None,
            });
        }
        let cooldown = Utc::now() + chrono::Duration::minutes(10);
        let _ = repo::user::set_captcha_cooldown(&self.pool, user_id, cooldown).await;
        let _ = repo::sudoku_captcha::delete_casino(&self.pool, captcha_id).await;
        Ok(CasinoSpinResult {
            win: false,
            captcha_token: None,
            cooldown_until: Some(cooldown),
        })
    }

    // ── Telegram login (called by the bot) ─────────────────────────────

    pub async fn login_via_telegram(&self, info: &TelegramUserInfo) -> AppResult<User> {
        match repo::user::get_by_telegram_id(&self.pool, info.telegram_id).await {
            Ok(u) => {
                if u.banned_at.is_some() {
                    return Err(AppError::UserBanned);
                }
                return Ok(u);
            }
            Err(AppError::UserNotFound) => {}
            Err(e) => return Err(e),
        }

        let username = self.generate_unique_username(info).await?;
        let random_password = random_hex(32);
        let hash = crypto::generate_hash(&random_password).map_err(AppError::Other)?;
        let input = CreateUser {
            username,
            password: random_password,
            hashed_password: hash,
            role: "user".into(),
            gender: None,
        };
        let mut user = repo::user::create(&self.pool, &input).await?;
        user = repo::user::set_telegram_id(&self.pool, user.id, info.telegram_id).await?;
        let display = build_display_name(&info.first_name, &info.last_name);
        if !display.is_empty() {
            match repo::user::update_settings(&self.pool, user.id, None, Some(&display), None).await {
                Ok(updated) => user = updated,
                Err(_) => {}
            }
        }
        Ok(user)
    }

    async fn generate_unique_username(&self, info: &TelegramUserInfo) -> AppResult<String> {
        if !info.username.is_empty() {
            let base = sanitize_username(&info.username);
            if !base.is_empty() {
                if let Ok(s) = self.find_available_username(&base).await {
                    return Ok(s);
                }
            }
        }
        if !info.first_name.is_empty() {
            let base = sanitize_username(&info.first_name);
            if !base.is_empty() {
                if let Ok(s) = self.find_available_username(&base).await {
                    return Ok(s);
                }
            }
        }
        Ok(format!("user_{}", random_hex(4)))
    }

    async fn find_available_username(&self, base: &str) -> AppResult<String> {
        let mut base = base.to_string();
        if base.len() < 3 {
            base.push_str("__");
        }
        if base.len() > 28 {
            base.truncate(28);
        }
        if self.is_username_available(&base).await {
            return Ok(base);
        }
        for i in 1..=99 {
            let candidate = format!("{base}_{i}");
            if candidate.len() <= 32 && self.is_username_available(&candidate).await {
                return Ok(candidate);
            }
        }
        let mut candidate = format!("{base}_{}", random_hex(4));
        if candidate.len() > 32 {
            candidate.truncate(32);
        }
        Ok(candidate)
    }

    async fn is_username_available(&self, username: &str) -> bool {
        matches!(
            repo::user::get_by_username(&self.pool, username).await,
            Err(AppError::UserNotFound)
        )
    }

    pub fn login_store(&self) -> Arc<LoginStore> {
        self.login_store.clone()
    }

    pub fn bot_username(&self) -> &str {
        &self.bot_username
    }

    pub fn jwt(&self) -> Arc<JwtManager> {
        self.jwt.clone()
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

fn sudoku_to_json(board: &[[u8; 9]; 9]) -> serde_json::Value {
    serde_json::to_value(board.iter().map(|r| r.to_vec()).collect::<Vec<_>>()).unwrap()
}

fn build_display_name(first: &str, last: &str) -> String {
    let parts: Vec<&str> = [first, last]
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    parts.join(" ")
}

fn random_hex(bytes: usize) -> String {
    use rand::RngCore;
    let mut buf = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

fn sanitize_username(raw: &str) -> String {
    let raw = raw.trim().to_lowercase();
    let mut s = String::with_capacity(raw.len());
    for c in raw.chars() {
        if c.is_ascii_alphabetic() {
            s.push(c);
        } else if c.is_ascii_digit() {
            s.push(c);
        } else if c == '_' {
            s.push(c);
        }
        // skip everything else (including non-ASCII letters, matching Go)
    }
    if s.is_empty() {
        return String::new();
    }
    if !s.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
        s.insert(0, 'u');
    }
    let pattern = regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap();
    if pattern.is_match(&s) {
        s
    } else {
        String::new()
    }
}

const _: Duration = Duration::from_secs(0); // keep `Duration` import grounded
