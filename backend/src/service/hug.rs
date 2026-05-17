//! Hug-related business logic.
//!
//! Wraps the hug repository with rules around cooldowns, intimacy decay,
//! streaks, captcha gating, daily rewards, and balance accounting. Callbacks
//! into the WebSocket hub and Telegram notifier live behind `RwLock<Option<_>>`
//! slots so the service can be wired up after construction (avoiding circular
//! dependencies with the bot/notifier).

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use parking_lot::RwLock;
use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::TtlCache;
use crate::error::{AppError, AppResult};
use crate::jwt::JwtManager;
use crate::models::{
    self, compute_intimacy_info, compute_streak_info, compute_streak_tier, compute_tier,
    is_hug_type_unlocked, slot_cost, valid_hug_type, Balance, BlockedUser, ConnectionItem,
    Hug, HugActivityItem, HugCooldown, HugDetail, HugFeedItem, IntimacyInfo,
    LeaderboardEntry, LeaderboardPairEntry, MutualHugStats, OutgoingPendingHug, PairStreak,
    PendingHugInboxItem, SlotInfo, StreakCalendarDay, StreakInfo, TopStreakEntry, User, UserStats,
    HUG_STATUS_PENDING, HUG_TYPE_STANDARD, MAX_HUG_SLOTS,
};
use crate::repo;

pub type HugCompletedCallback =
    Arc<dyn Fn(&HugFeedItem, i32, &Option<String>) + Send + Sync>;
pub type HugSuggestionCallback =
    Arc<dyn Fn(Uuid, &PendingHugInboxItem, &Option<String>) + Send + Sync>;
pub type HugDeclinedCallback = Arc<dyn Fn(Uuid, Uuid, Uuid) + Send + Sync>;
pub type HugCancelledCallback = Arc<dyn Fn(Uuid, Uuid) + Send + Sync>;

const DEFAULT_COOLDOWN_SECONDS: i32 = 3600;
const COOLDOWN_REDUCTION_PER_UPGRADE: i32 = 600;
const UPGRADE_COST: i32 = 5;
const DECLINE_COOLDOWN_SECONDS: i64 = 300;

pub struct HugService {
    pool: PgPool,
    jwt: Arc<JwtManager>,
    leaderboard_cache: TtlCache<String, Vec<LeaderboardEntry>>,
    activity_cache: TtlCache<String, Vec<HugActivityItem>>,

    on_completed: RwLock<Option<HugCompletedCallback>>,
    on_suggestion: RwLock<Option<HugSuggestionCallback>>,
    on_declined: RwLock<Option<HugDeclinedCallback>>,
    on_cancelled: RwLock<Option<HugCancelledCallback>>,
}

#[derive(Debug, Clone)]
pub struct CooldownInfo {
    pub cooldown: HugCooldown,
    pub remaining_seconds: i32,
    pub can_hug: bool,
    pub decline_remaining: i32,
    pub effective_cooldown: i32,
    pub intimacy_reduction_pct: i32,
}

impl HugService {
    pub fn new(pool: PgPool, jwt: Arc<JwtManager>) -> Self {
        Self {
            pool,
            jwt,
            leaderboard_cache: TtlCache::new(Duration::from_secs(30)),
            activity_cache: TtlCache::new(Duration::from_secs(120)),
            on_completed: RwLock::new(None),
            on_suggestion: RwLock::new(None),
            on_declined: RwLock::new(None),
            on_cancelled: RwLock::new(None),
        }
    }

    pub fn set_on_completed(&self, cb: HugCompletedCallback) {
        *self.on_completed.write() = Some(cb);
    }
    pub fn set_on_suggestion(&self, cb: HugSuggestionCallback) {
        *self.on_suggestion.write() = Some(cb);
    }
    pub fn set_on_declined(&self, cb: HugDeclinedCallback) {
        *self.on_declined.write() = Some(cb);
    }
    pub fn set_on_cancelled(&self, cb: HugCancelledCallback) {
        *self.on_cancelled.write() = Some(cb);
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // ── flow: suggest / accept / decline / cancel ─────────────────────

    pub async fn suggest_hug(
        &self,
        giver_id: Uuid,
        receiver_id: Uuid,
        hug_type: &str,
        comment: Option<&str>,
        captcha_token: Option<&str>,
    ) -> AppResult<(Hug, User)> {
        if giver_id == receiver_id {
            return Err(AppError::CannotHugSelf);
        }

        let hug_type = if hug_type.is_empty() {
            HUG_TYPE_STANDARD
        } else {
            hug_type
        };
        if !valid_hug_type(hug_type) {
            return Err(AppError::HugTypeLocked);
        }

        if repo::block::is_blocked_by_either(&self.pool, giver_id, receiver_id).await? {
            return Err(AppError::UserBlocked);
        }

        let receiver = repo::user::get_by_id(&self.pool, receiver_id).await?;
        let giver = repo::user::get_by_id(&self.pool, giver_id).await?;

        if giver.captcha_type != "none" {
            let token = captcha_token.unwrap_or("").trim();
            if token.is_empty() {
                return Err(AppError::CaptchaRequired);
            }
            let token_user_id = self
                .jwt
                .parse_captcha_token(token)
                .map_err(|_| AppError::CaptchaFailed)?;
            if token_user_id != giver_id {
                return Err(AppError::CaptchaFailed);
            }
        }

        if hug_type != HUG_TYPE_STANDARD {
            let pair = repo::intimacy::get_pair(&self.pool, giver_id, receiver_id).await?;
            let raw_score = pair.map(|p| p.raw_score).unwrap_or(0);
            if !is_hug_type_unlocked(raw_score, hug_type) {
                return Err(AppError::HugTypeLocked);
            }
        }

        let mut tx = self.pool.begin().await?;
        let (outgoing, pair_pending, reverse_pending) =
            repo::hug::check_suggest_eligibility(&mut *tx, giver_id, receiver_id).await?;
        let slots = repo::user::get_slots(&mut *tx, giver_id).await?;
        if outgoing >= slots {
            let _ = tx.rollback().await;
            return Err(AppError::AlreadyHasPendingHug);
        }
        if pair_pending {
            let _ = tx.rollback().await;
            return Err(AppError::PendingHugExists);
        }
        if reverse_pending {
            let _ = tx.rollback().await;
            return Err(AppError::ReversePendingHugExists);
        }

        let cooldown = repo::hug::get_cooldown(&mut *tx, giver_id, receiver_id).await?;
        if let Some(ref cd) = cooldown {
            if let Some(until) = cd.decline_cooldown_until {
                if until > Utc::now() {
                    let _ = tx.rollback().await;
                    return Err(AppError::DeclineCooldownActive);
                }
            }
            let mut effective = cd.cooldown_seconds;
            let intimacy_pair = repo::intimacy::get_pair(&mut *tx, giver_id, receiver_id).await?;
            if let Some(pair) = intimacy_pair {
                let tier = compute_tier(pair.raw_score);
                let reduction = (effective as f64) * tier.cooldown_reduction;
                effective -= reduction as i32;
                if effective < 0 {
                    effective = 0;
                }
            }
            let elapsed = Utc::now()
                .signed_duration_since(cd.last_hug_at)
                .num_seconds();
            if elapsed < effective as i64 {
                let _ = tx.rollback().await;
                return Err(AppError::HugCooldownActive);
            }
        }

        let hug = repo::hug::insert(
            &mut *tx,
            giver_id,
            receiver_id,
            HUG_STATUS_PENDING,
            hug_type,
            comment,
        )
        .await?;
        tx.commit().await?;

        if let Some(cb) = self.on_suggestion.read().clone() {
            let item = PendingHugInboxItem {
                id: hug.id,
                giver_id: hug.giver_id,
                receiver_id: hug.receiver_id,
                giver_username: giver.username.clone(),
                giver_gender: giver.gender.clone(),
                giver_display_name: giver.display_name.clone(),
                hug_type: hug.hug_type.clone(),
                comment: hug.comment.clone(),
                created_at: hug.created_at,
            };
            cb(receiver_id, &item, &hug.comment);
        }

        Ok((hug, receiver))
    }

    pub async fn accept_hug(&self, hug_id: Uuid, receiver_id: Uuid) -> AppResult<Hug> {
        let mut tx = self.pool.begin().await?;
        let existing = repo::hug::get_by_id(&mut *tx, hug_id).await?;
        let Some(existing) = existing else {
            let _ = tx.rollback().await;
            return Err(AppError::HugNotFound);
        };
        if existing.status != HUG_STATUS_PENDING {
            let _ = tx.rollback().await;
            return Err(AppError::HugNotPending);
        }

        let tier_key =
            compute_and_update_streak(&mut tx, existing.giver_id, existing.receiver_id).await;

        let accepted = repo::hug::accept(&mut *tx, hug_id, receiver_id, &tier_key).await?;
        let Some(hug) = accepted else {
            let _ = tx.rollback().await;
            return Err(AppError::HugExpired);
        };

        let intimacy = repo::intimacy::upsert_pair(&mut *tx, hug.giver_id, hug.receiver_id).await?;
        let tier = compute_tier(intimacy.raw_score);
        let bonus = tier.bonus_coins;

        if hug.comment.is_none() {
            repo::balance::add(&mut *tx, hug.giver_id, 1 + bonus).await?;
        }
        repo::balance::add(&mut *tx, hug.receiver_id, 1 + bonus).await?;

        repo::hug::upsert_cooldown(
            &mut *tx,
            hug.giver_id,
            hug.receiver_id,
            DEFAULT_COOLDOWN_SECONDS,
        )
        .await?;

        tx.commit().await?;

        self.leaderboard_cache.invalidate_all();

        if let Some(cb) = self.on_completed.read().clone() {
            let pool = self.pool.clone();
            let hug_id = hug.id;
            let giver = hug.giver_id;
            let receiver = hug.receiver_id;
            let hug_type = hug.hug_type.clone();
            let streak_tier = hug.streak_tier.clone();
            let created_at = hug.accepted_at.unwrap_or(hug.created_at);
            let has_comment = hug.comment.is_some();
            let comment = hug.comment.clone();
            tokio::spawn(async move {
                // Look up giver/receiver usernames/display names for the broadcast payload.
                let giver_u = repo::user::get_by_id(&pool, giver).await.ok();
                let receiver_u = repo::user::get_by_id(&pool, receiver).await.ok();
                let giver_username = giver_u.as_ref().map(|u| u.username.clone()).unwrap_or_default();
                let receiver_username = receiver_u
                    .as_ref()
                    .map(|u| u.username.clone())
                    .unwrap_or_default();
                let giver_gender = giver_u.as_ref().and_then(|u| u.gender.clone());
                let giver_display = giver_u.as_ref().and_then(|u| u.display_name.clone());
                let receiver_display = receiver_u.as_ref().and_then(|u| u.display_name.clone());
                let feed_item = HugFeedItem {
                    id: hug_id,
                    giver_id: giver,
                    receiver_id: receiver,
                    giver_username,
                    receiver_username,
                    giver_gender,
                    giver_display_name: giver_display,
                    receiver_display_name: receiver_display,
                    hug_type,
                    has_comment,
                    streak_tier,
                    created_at,
                };
                cb(&feed_item, bonus, &comment);
            });
        }

        Ok(hug)
    }

    pub async fn decline_hug(&self, hug_id: Uuid, receiver_id: Uuid) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        let declined = repo::hug::decline(&mut *tx, hug_id, receiver_id).await?;
        let hug = match declined {
            Some(h) => h,
            None => {
                let existing = repo::hug::get_by_id(&mut *tx, hug_id).await?;
                let _ = tx.rollback().await;
                return Err(match existing {
                    None => AppError::HugNotFound,
                    Some(h) if h.status != HUG_STATUS_PENDING => AppError::HugNotPending,
                    Some(_) => AppError::HugExpired,
                });
            }
        };

        let until = Utc::now() + chrono::Duration::seconds(DECLINE_COOLDOWN_SECONDS);
        repo::hug::set_decline_cooldown(&mut *tx, hug.giver_id, hug.receiver_id, until).await?;
        tx.commit().await?;

        if let Some(cb) = self.on_declined.read().clone() {
            cb(hug.giver_id, hug_id, hug.receiver_id);
        }
        Ok(())
    }

    pub async fn cancel_hug(&self, hug_id: Uuid, giver_id: Uuid) -> AppResult<()> {
        let hug = repo::hug::cancel(&self.pool, hug_id, giver_id).await?;
        let hug = match hug {
            Some(h) => h,
            None => {
                let existing = repo::hug::get_by_id(&self.pool, hug_id).await?;
                return Err(match existing {
                    None => AppError::HugNotFound,
                    Some(h) if h.status != HUG_STATUS_PENDING => AppError::HugNotPending,
                    Some(_) => AppError::HugExpired,
                });
            }
        };
        if let Some(cb) = self.on_cancelled.read().clone() {
            cb(hug.receiver_id, hug_id);
        }
        Ok(())
    }

    pub async fn get_detail(
        &self,
        hug_id: Uuid,
        requester_id: Uuid,
        is_admin: bool,
    ) -> AppResult<HugDetail> {
        let detail = repo::hug::detail(&self.pool, hug_id)
            .await?
            .ok_or(AppError::HugNotFound)?;
        if !is_admin && detail.giver_id != requester_id && detail.receiver_id != requester_id {
            // Treat as not-found for privacy.
            return Err(AppError::HugNotFound);
        }
        Ok(detail)
    }

    pub async fn get_history(
        &self,
        user_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> AppResult<Vec<HugFeedItem>> {
        repo::hug::list_for_user(&self.pool, user_id, limit, offset).await
    }

    pub async fn get_recent_feed(
        &self,
        limit: i32,
        offset: i32,
    ) -> AppResult<Vec<HugFeedItem>> {
        repo::hug::recent_feed(&self.pool, limit, offset).await
    }

    pub async fn get_activity(&self) -> AppResult<Vec<HugActivityItem>> {
        const KEY: &str = "activity";
        if let Some(cached) = self.activity_cache.get(&KEY.to_string()) {
            return Ok(cached);
        }
        let items = repo::hug::activity(&self.pool).await?;
        self.activity_cache.set(KEY.to_string(), items.clone());
        Ok(items)
    }

    pub async fn get_leaderboard(
        &self,
        limit: i32,
        offset: i32,
    ) -> AppResult<Vec<LeaderboardEntry>> {
        let key = format!("{limit}:{offset}");
        if let Some(cached) = self.leaderboard_cache.get(&key) {
            return Ok(cached);
        }
        let entries = repo::hug::leaderboard(&self.pool, limit, offset).await?;
        self.leaderboard_cache.set(key, entries.clone());
        Ok(entries)
    }

    pub async fn get_user_stats(
        &self,
        user_id: Uuid,
        gender: Option<&str>,
    ) -> AppResult<UserStats> {
        repo::hug::user_stats(&self.pool, user_id, gender).await
    }

    pub async fn get_user_profile(
        &self,
        user_id: Uuid,
        viewer_id: Option<Uuid>,
    ) -> AppResult<UserProfileBundle> {
        let user = repo::user::get_by_id(&self.pool, user_id).await?;
        let stats = repo::hug::user_stats(&self.pool, user_id, user.gender.as_deref()).await?;
        let balance = repo::balance::get(&self.pool, user_id).await.ok();

        let mut mutual = None;
        let mut is_blocked = false;
        let mut intimacy = None;
        if let Some(viewer) = viewer_id {
            if viewer != user_id {
                mutual = repo::hug::count_mutual(&self.pool, user_id, viewer).await.ok();
                is_blocked = repo::block::is_blocked_by_either(&self.pool, viewer, user_id)
                    .await
                    .unwrap_or(false);
                let pair = repo::intimacy::get_pair(&self.pool, viewer, user_id)
                    .await
                    .ok()
                    .flatten();
                let raw = pair.map(|p| p.raw_score).unwrap_or(0);
                intimacy = Some(compute_intimacy_info(raw));
            }
        }
        Ok(UserProfileBundle {
            user,
            stats,
            balance,
            mutual,
            is_blocked,
            intimacy,
        })
    }

    pub async fn search_users(
        &self,
        query: &str,
        viewer_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> AppResult<Vec<User>> {
        repo::user::search_users(&self.pool, query, viewer_id, limit, offset).await
    }

    pub async fn block_user(&self, blocker: Uuid, blocked: Uuid) -> AppResult<()> {
        if blocker == blocked {
            return Err(AppError::CannotBlockSelf);
        }
        let _ = repo::user::get_by_id(&self.pool, blocked).await?;
        repo::block::block(&self.pool, blocker, blocked).await
    }

    pub async fn unblock_user(&self, blocker: Uuid, blocked: Uuid) -> AppResult<()> {
        repo::block::unblock(&self.pool, blocker, blocked).await
    }

    pub async fn blocked_users(&self, user_id: Uuid) -> AppResult<Vec<BlockedUser>> {
        repo::block::list_blocked(&self.pool, user_id).await
    }

    pub async fn expire_pending_hugs(&self) -> AppResult<()> {
        repo::hug::expire_pending(&self.pool).await
    }

    pub async fn pending_inbox(&self, user_id: Uuid) -> AppResult<Vec<PendingHugInboxItem>> {
        repo::hug::pending_inbox(&self.pool, user_id).await
    }

    pub async fn inbox_count(&self, user_id: Uuid) -> AppResult<i64> {
        repo::hug::count_pending(&self.pool, user_id).await
    }

    pub async fn outgoing_hugs(
        &self,
        user_id: Uuid,
    ) -> AppResult<(Vec<OutgoingPendingHug>, SlotInfo)> {
        let hugs = repo::hug::outgoing_pending(&self.pool, user_id).await?;
        let slots = repo::user::get_slots(&self.pool, user_id).await?;
        let mut info = SlotInfo {
            total_slots: slots,
            used_slots: hugs.len() as i32,
            next_slot_cost: None,
        };
        if slots < MAX_HUG_SLOTS {
            info.next_slot_cost = Some(slot_cost(slots + 1));
        }
        Ok((hugs, info))
    }

    pub async fn buy_hug_slot(&self, user_id: Uuid) -> AppResult<(SlotInfo, i32)> {
        let mut tx = self.pool.begin().await?;
        let current = repo::user::get_slots(&mut *tx, user_id).await?;
        if current >= MAX_HUG_SLOTS {
            let _ = tx.rollback().await;
            return Err(AppError::MaxSlotsReached);
        }
        let cost = slot_cost(current + 1);
        let bal = repo::balance::deduct(&mut *tx, user_id, cost).await?;
        let Some(bal) = bal else {
            let _ = tx.rollback().await;
            return Err(AppError::InsufficientBalance);
        };
        let new_slots = repo::user::increment_slots(&mut *tx, user_id).await?;
        let Some(new_slots) = new_slots else {
            let _ = tx.rollback().await;
            return Err(AppError::MaxSlotsReached);
        };
        let outgoing = repo::hug::outgoing_pending(&mut *tx, user_id).await?;
        tx.commit().await?;
        let mut info = SlotInfo {
            total_slots: new_slots,
            used_slots: outgoing.len() as i32,
            next_slot_cost: None,
        };
        if new_slots < MAX_HUG_SLOTS {
            info.next_slot_cost = Some(slot_cost(new_slots + 1));
        }
        Ok((info, bal.amount))
    }

    // ── balance + daily reward ─────────────────────────────────────────

    pub async fn get_balance(&self, user_id: Uuid) -> AppResult<Balance> {
        repo::balance::get(&self.pool, user_id).await
    }

    pub async fn claim_daily_reward(
        &self,
        user_id: Uuid,
    ) -> AppResult<(i32, i32, i32, bool)> {
        // Returns (amount, streak_days, new_balance, already_claimed).
        let mut tx = self.pool.begin().await?;
        let existing = repo::daily_reward::get(&mut *tx, user_id).await?;
        let today = models::today_ymd();
        if let Some(ref e) = existing {
            if e.last_claimed_at.format("%Y-%m-%d").to_string() == today {
                let bal = repo::balance::get(&mut *tx, user_id).await?;
                tx.commit().await?;
                return Ok((0, e.streak_days, bal.amount, true));
            }
        }
        let reward = repo::daily_reward::claim(&mut *tx, user_id).await?;
        let bonus = (reward.streak_days - 1).min(5);
        let amount = 5 + bonus;
        let bal = repo::balance::add(&mut *tx, user_id, amount).await?;
        tx.commit().await?;
        Ok((amount, reward.streak_days, bal.amount, false))
    }

    pub async fn daily_reward_status(
        &self,
        user_id: Uuid,
    ) -> AppResult<(bool, chrono::DateTime<chrono::Utc>, i32, Option<chrono::DateTime<chrono::Utc>>)> {
        let existing = repo::daily_reward::get(&self.pool, user_id).await?;
        let now = chrono::Utc::now();
        let Some(reward) = existing else {
            return Ok((true, now, 0, None));
        };
        let last = reward.last_claimed_at;
        let today = models::today_ymd();
        let can_claim = last.format("%Y-%m-%d").to_string() != today;
        let next = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            last.date_naive().and_hms_opt(0, 0, 0).unwrap(),
            chrono::Utc,
        ) + chrono::Duration::days(1);
        let next_claim_at = if can_claim { now } else { next };
        Ok((can_claim, next_claim_at, reward.streak_days, Some(last)))
    }

    // ── cooldowns / upgrade ────────────────────────────────────────────

    pub async fn cooldown_info(&self, user_a: Uuid, user_b: Uuid) -> AppResult<CooldownInfo> {
        let cooldown = repo::hug::get_cooldown(&self.pool, user_a, user_b).await?;
        let intimacy_pair = repo::intimacy::get_pair(&self.pool, user_a, user_b).await?;
        let reduction_pct = intimacy_pair
            .map(|p| (compute_tier(p.raw_score).cooldown_reduction * 100.0) as i32)
            .unwrap_or(0);

        if cooldown.is_none() {
            let base = DEFAULT_COOLDOWN_SECONDS;
            let effective = base
                - ((base as f64) * (reduction_pct as f64) / 100.0) as i32;
            let effective = if effective < 0 { 0 } else { effective };
            return Ok(CooldownInfo {
                cooldown: HugCooldown {
                    user_a_id: user_a,
                    user_b_id: user_b,
                    last_hug_at: Utc::now(),
                    cooldown_seconds: base,
                    decline_cooldown_until: None,
                },
                remaining_seconds: 0,
                can_hug: true,
                decline_remaining: 0,
                effective_cooldown: effective,
                intimacy_reduction_pct: reduction_pct,
            });
        }
        let cd = cooldown.unwrap();
        let mut effective = cd.cooldown_seconds;
        let reduction = (effective as f64) * (reduction_pct as f64) / 100.0;
        effective -= reduction as i32;
        if effective < 0 {
            effective = 0;
        }

        let elapsed = Utc::now()
            .signed_duration_since(cd.last_hug_at)
            .num_seconds() as i32;
        let mut remaining = effective - elapsed;
        if remaining < 0 {
            remaining = 0;
        }
        let mut can_hug = remaining <= 0;
        let mut decline_remaining = 0;
        if let Some(until) = cd.decline_cooldown_until {
            let dr = until.signed_duration_since(Utc::now()).num_seconds();
            if dr > 0 {
                decline_remaining = dr as i32;
                can_hug = false;
            }
        }
        Ok(CooldownInfo {
            cooldown: cd,
            remaining_seconds: remaining,
            can_hug,
            decline_remaining,
            effective_cooldown: effective,
            intimacy_reduction_pct: reduction_pct,
        })
    }

    pub async fn upgrade_cooldown(
        &self,
        payer_id: Uuid,
        other_user_id: Uuid,
    ) -> AppResult<HugCooldown> {
        let mut tx = self.pool.begin().await?;
        let bal = repo::balance::deduct(&mut *tx, payer_id, UPGRADE_COST).await?;
        if bal.is_none() {
            let _ = tx.rollback().await;
            return Err(AppError::InsufficientBalance);
        }
        let existing = repo::hug::get_cooldown(&mut *tx, payer_id, other_user_id).await?;
        if existing.is_none() {
            repo::hug::upsert_cooldown(
                &mut *tx,
                payer_id,
                other_user_id,
                DEFAULT_COOLDOWN_SECONDS,
            )
            .await?;
        }
        let reduced = repo::hug::reduce_cooldown(
            &mut *tx,
            payer_id,
            other_user_id,
            COOLDOWN_REDUCTION_PER_UPGRADE,
        )
        .await?;
        let Some(reduced) = reduced else {
            let _ = tx.rollback().await;
            return Err(AppError::CooldownNotFound);
        };
        tx.commit().await?;
        Ok(reduced)
    }

    // ── intimacy / streaks ─────────────────────────────────────────────

    pub async fn pair_intimacy(&self, user_a: Uuid, user_b: Uuid) -> AppResult<IntimacyInfo> {
        let pair = repo::intimacy::get_pair(&self.pool, user_a, user_b).await?;
        Ok(compute_intimacy_info(pair.map(|p| p.raw_score).unwrap_or(0)))
    }

    pub async fn user_connections(
        &self,
        user_id: Uuid,
        limit: i32,
        offset: i32,
    ) -> AppResult<Vec<ConnectionItem>> {
        repo::intimacy::user_connections(&self.pool, user_id, limit, offset).await
    }

    pub async fn intimacy_leaderboard(
        &self,
        limit: i32,
        offset: i32,
    ) -> AppResult<Vec<LeaderboardPairEntry>> {
        repo::intimacy::leaderboard(&self.pool, limit, offset).await
    }

    pub async fn apply_intimacy_decay(&self) -> AppResult<()> {
        repo::intimacy::apply_decay(&self.pool).await
    }

    pub async fn pair_streak(&self, user_a: Uuid, user_b: Uuid) -> AppResult<StreakInfo> {
        let streak = repo::hug::get_pair_streak(&self.pool, user_a, user_b).await?;
        Ok(compute_streak_info(streak.as_ref()))
    }

    pub async fn user_top_streaks(
        &self,
        user_id: Uuid,
        limit: i32,
    ) -> AppResult<Vec<TopStreakEntry>> {
        repo::hug::user_top_streaks(&self.pool, user_id, limit).await
    }

    pub async fn pair_streak_calendar(
        &self,
        user_a: Uuid,
        user_b: Uuid,
    ) -> AppResult<Vec<StreakCalendarDay>> {
        let since = Utc::now() - chrono::Duration::days(90);
        repo::hug::pair_streak_calendar(&self.pool, user_a, user_b, since).await
    }
}

pub struct UserProfileBundle {
    pub user: User,
    pub stats: UserStats,
    pub balance: Option<Balance>,
    pub mutual: Option<MutualHugStats>,
    pub is_blocked: bool,
    pub intimacy: Option<IntimacyInfo>,
}

async fn compute_and_update_streak(
    tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
    giver_id: Uuid,
    receiver_id: Uuid,
) -> String {
    let today = Utc::now().date_naive();
    let streak_existing = match repo::hug::get_pair_streak(&mut **tx, giver_id, receiver_id).await {
        Ok(s) => s,
        Err(_) => return String::new(),
    };

    let (user_a_id, user_b_id) = if giver_id < receiver_id {
        (giver_id, receiver_id)
    } else {
        (receiver_id, giver_id)
    };
    let giver_is_a = giver_id == user_a_id;

    let mut streak = streak_existing.unwrap_or_else(|| PairStreak {
        user_a_id,
        user_b_id,
        current_streak: 0,
        best_streak: 0,
        last_streak_date: None,
        a_hugged_today: false,
        b_hugged_today: false,
        today_date: today,
    });

    if streak.today_date != today {
        let yesterday = today.pred_opt().unwrap_or(today);
        let prev_day_completed = streak.a_hugged_today && streak.b_hugged_today;
        let continues = (prev_day_completed && streak.today_date == yesterday)
            || streak
                .last_streak_date
                .map(|d| d == yesterday || d == today)
                .unwrap_or(false);
        if !continues {
            streak.current_streak = 0;
        }
        streak.a_hugged_today = false;
        streak.b_hugged_today = false;
        streak.today_date = today;
    }

    if giver_is_a {
        streak.a_hugged_today = true;
    } else {
        streak.b_hugged_today = true;
    }

    if streak.a_hugged_today && streak.b_hugged_today {
        let already = streak.last_streak_date.map(|d| d == today).unwrap_or(false);
        if !already {
            streak.current_streak += 1;
            streak.last_streak_date = Some(today);
            if streak.current_streak > streak.best_streak {
                streak.best_streak = streak.current_streak;
            }
        }
    }

    match repo::hug::upsert_pair_streak(&mut **tx, &streak).await {
        Ok(_) => compute_streak_tier(streak.current_streak).key.to_string(),
        Err(_) => String::new(),
    }
}
