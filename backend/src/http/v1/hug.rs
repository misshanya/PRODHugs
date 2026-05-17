//! Hug-related and profile endpoints.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, ErrorCode};
use crate::http::auth::AuthUser;
use crate::http::dto::{
    BlockedUserDto, ConnectionItemDto, HugActivityItemDto, HugFeedItemDto, IntimacyInfoDto,
    IntimacyLeaderboardEntryDto, LeaderboardEntryDto, OutgoingPendingHugDto, PendingHugInboxItemDto,
    SlotInfoDto, StreakCalendarDayDto, StreakInfoDto, TopStreakEntryDto, UserListItemDto,
};
use crate::http::error::{ApiError, HttpResult};
use crate::http::AppState;
use crate::models;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

fn paginated(p: &Pagination, default_limit: i32) -> (i32, i32) {
    let limit = p.limit.filter(|v| *v > 0).unwrap_or(default_limit);
    let offset = p.offset.filter(|v| *v >= 0).unwrap_or(0);
    (limit, offset)
}

// ── hug suggest / accept / decline / cancel ───────────────────────────

#[derive(Debug, Deserialize, Default)]
pub struct SuggestHugReq {
    pub hug_type: Option<String>,
    pub comment: Option<String>,
    pub captcha_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HugResp {
    pub id: Uuid,
    pub giver_id: Uuid,
    pub receiver_id: Uuid,
    pub status: String,
    pub hug_type: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_gender: Option<String>,
}

pub async fn suggest_hug(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(receiver_id): Path<Uuid>,
    body: Option<Json<SuggestHugReq>>,
) -> HttpResult<(StatusCode, Json<HugResp>)> {
    if auth.user_id == receiver_id {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::CannotHugSelf,
            "You cannot hug yourself",
        ));
    }
    let req = body.map(|Json(r)| r).unwrap_or_default();
    let hug_type = req.hug_type.unwrap_or_else(|| "standard".into());

    let comment = req
        .comment
        .as_deref()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    if let Some(ref c) = comment {
        if c.chars().count() > 140 {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::CommentTooLong,
                "Comment must be at most 140 characters",
            ));
        }
    }
    let captcha = req.captcha_token.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty());

    let (hug, receiver) = state
        .hug
        .suggest_hug(auth.user_id, receiver_id, &hug_type, comment.as_deref(), captcha)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(HugResp {
            id: hug.id,
            giver_id: hug.giver_id,
            receiver_id: hug.receiver_id,
            status: hug.status,
            hug_type: hug.hug_type,
            created_at: hug.created_at,
            accepted_at: hug.accepted_at,
            comment: hug.comment,
            receiver_username: Some(receiver.username),
            receiver_gender: receiver.gender,
        }),
    ))
}

pub async fn get_detail(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(hug_id): Path<Uuid>,
) -> HttpResult<Json<serde_json::Value>> {
    let detail = state
        .hug
        .get_detail(hug_id, auth.user_id, auth.is_admin())
        .await?;
    let json = serde_json::json!({
        "id": detail.id,
        "giver_id": detail.giver_id,
        "receiver_id": detail.receiver_id,
        "giver_username": detail.giver_username,
        "receiver_username": detail.receiver_username,
        "giver_gender": detail.giver_gender,
        "giver_display_name": detail.giver_display_name,
        "receiver_display_name": detail.receiver_display_name,
        "status": detail.status,
        "hug_type": detail.hug_type,
        "comment": detail.comment,
        "streak_tier": detail.streak_tier,
        "created_at": detail.created_at,
        "accepted_at": detail.accepted_at,
    });
    Ok(Json(json))
}

pub async fn accept_hug(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(hug_id): Path<Uuid>,
) -> HttpResult<Json<HugResp>> {
    let hug = state.hug.accept_hug(hug_id, auth.user_id).await?;
    Ok(Json(HugResp {
        id: hug.id,
        giver_id: hug.giver_id,
        receiver_id: hug.receiver_id,
        status: hug.status,
        hug_type: hug.hug_type,
        created_at: hug.created_at,
        accepted_at: hug.accepted_at,
        comment: hug.comment,
        receiver_username: None,
        receiver_gender: None,
    }))
}

pub async fn decline_hug(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(hug_id): Path<Uuid>,
) -> HttpResult<Json<serde_json::Value>> {
    state.hug.decline_hug(hug_id, auth.user_id).await?;
    Ok(Json(serde_json::json!({ "message": "Hug declined" })))
}

pub async fn cancel_hug(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(hug_id): Path<Uuid>,
) -> HttpResult<Json<serde_json::Value>> {
    state.hug.cancel_hug(hug_id, auth.user_id).await?;
    Ok(Json(serde_json::json!({ "message": "Hug cancelled" })))
}

// ── feed / leaderboard / activity / history ───────────────────────────

pub async fn feed(
    State(state): State<Arc<AppState>>,
    Query(p): Query<Pagination>,
    _auth: AuthUser,
) -> HttpResult<Json<Vec<HugFeedItemDto>>> {
    let (limit, offset) = paginated(&p, 50);
    let items = state.hug.get_recent_feed(limit, offset).await?;
    Ok(Json(items.iter().map(HugFeedItemDto::from).collect()))
}

pub async fn history(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<Vec<HugFeedItemDto>>> {
    let items = state.hug.get_history(auth.user_id, 100, 0).await?;
    Ok(Json(items.iter().map(HugFeedItemDto::from).collect()))
}

pub async fn activity(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> HttpResult<Json<Vec<HugActivityItemDto>>> {
    let items = state.hug.get_activity().await?;
    Ok(Json(items.iter().map(HugActivityItemDto::from).collect()))
}

pub async fn leaderboard(
    State(state): State<Arc<AppState>>,
    Query(p): Query<Pagination>,
    _auth: AuthUser,
) -> HttpResult<Json<Vec<LeaderboardEntryDto>>> {
    let (limit, offset) = paginated(&p, 20);
    let entries = state.hug.get_leaderboard(limit, offset).await?;
    Ok(Json(entries.iter().map(LeaderboardEntryDto::from).collect()))
}

// ── inbox / outgoing / slots ──────────────────────────────────────────

pub async fn inbox(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<Vec<PendingHugInboxItemDto>>> {
    let items = state.hug.pending_inbox(auth.user_id).await?;
    Ok(Json(items.iter().map(PendingHugInboxItemDto::from).collect()))
}

pub async fn inbox_count(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<serde_json::Value>> {
    let c = state.hug.inbox_count(auth.user_id).await?;
    Ok(Json(serde_json::json!({ "count": c })))
}

#[derive(Debug, Serialize)]
pub struct OutgoingResp {
    pub hugs: Vec<OutgoingPendingHugDto>,
    pub slots: SlotInfoDto,
}

pub async fn outgoing(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<OutgoingResp>> {
    let (hugs, slots) = state.hug.outgoing_hugs(auth.user_id).await?;
    Ok(Json(OutgoingResp {
        hugs: hugs.iter().map(OutgoingPendingHugDto::from).collect(),
        slots: SlotInfoDto::from(&slots),
    }))
}

#[derive(Debug, Serialize)]
pub struct BuySlotResp {
    pub slots: SlotInfoDto,
    pub new_balance: i32,
}

pub async fn buy_slot(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<BuySlotResp>> {
    let (slots, new_balance) = state.hug.buy_hug_slot(auth.user_id).await?;
    Ok(Json(BuySlotResp {
        slots: SlotInfoDto::from(&slots),
        new_balance,
    }))
}

// ── cooldown ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CooldownResp {
    pub user_a_id: Uuid,
    pub user_b_id: Uuid,
    pub cooldown_seconds: i32,
    pub remaining_seconds: i32,
    pub can_hug: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decline_cooldown_remaining: Option<i32>,
    pub effective_cooldown_seconds: i32,
    pub intimacy_reduction_pct: i32,
}

pub async fn get_cooldown(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(user_b): Path<Uuid>,
) -> HttpResult<Json<CooldownResp>> {
    let info = state.hug.cooldown_info(auth.user_id, user_b).await?;
    let mut decline = None;
    let mut remaining = info.remaining_seconds;
    if info.decline_remaining > 0 {
        decline = Some(info.decline_remaining);
        if info.decline_remaining > remaining {
            remaining = info.decline_remaining;
        }
    }
    Ok(Json(CooldownResp {
        user_a_id: info.cooldown.user_a_id,
        user_b_id: info.cooldown.user_b_id,
        cooldown_seconds: info.cooldown.cooldown_seconds,
        remaining_seconds: remaining,
        can_hug: info.can_hug,
        decline_cooldown_remaining: decline,
        effective_cooldown_seconds: info.effective_cooldown,
        intimacy_reduction_pct: info.intimacy_reduction_pct,
    }))
}

#[derive(Debug, Serialize)]
pub struct UpgradeCooldownResp {
    pub user_a_id: Uuid,
    pub user_b_id: Uuid,
    pub cooldown_seconds: i32,
    pub remaining_seconds: i32,
    pub can_hug: bool,
}

pub async fn upgrade_cooldown(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(other): Path<Uuid>,
) -> HttpResult<Json<UpgradeCooldownResp>> {
    let cd = state.hug.upgrade_cooldown(auth.user_id, other).await?;
    let elapsed = Utc::now().signed_duration_since(cd.last_hug_at).num_seconds() as i32;
    let mut remaining = cd.cooldown_seconds - elapsed;
    if remaining < 0 {
        remaining = 0;
    }
    Ok(Json(UpgradeCooldownResp {
        user_a_id: cd.user_a_id,
        user_b_id: cd.user_b_id,
        cooldown_seconds: cd.cooldown_seconds,
        remaining_seconds: remaining,
        can_hug: remaining <= 0,
    }))
}

// ── balance + daily ───────────────────────────────────────────────────

pub async fn balance(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<serde_json::Value>> {
    let bal = state.hug.get_balance(auth.user_id).await?;
    Ok(Json(serde_json::json!({
        "user_id": bal.user_id,
        "amount": bal.amount,
    })))
}

#[derive(Debug, Serialize)]
pub struct DailyResp {
    pub amount: i32,
    pub streak_days: i32,
    pub new_balance: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub already_claimed: Option<bool>,
}

pub async fn claim_daily(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<DailyResp>> {
    let (amount, streak, new_balance, already) = state.hug.claim_daily_reward(auth.user_id).await?;
    Ok(Json(DailyResp {
        amount,
        streak_days: streak,
        new_balance,
        already_claimed: Some(already),
    }))
}

// ── profile / search / connections / intimacy / streak ────────────────

pub async fn user_profile(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    auth: AuthUser,
) -> HttpResult<Json<serde_json::Value>> {
    let bundle = state
        .hug
        .get_user_profile(user_id, Some(auth.user_id))
        .await?;
    let mut profile = serde_json::json!({
        "id": bundle.user.id,
        "username": bundle.user.username,
        "role": bundle.user.role,
        "gender": bundle.user.gender,
        "display_name": bundle.user.display_name,
        "tag": bundle.user.tag,
        "special_tag": bundle.user.special_tag,
        "hugs_given": bundle.stats.hugs_given,
        "hugs_received": bundle.stats.hugs_received,
        "total_hugs": bundle.stats.total_hugs,
        "rank": models::get_rank(bundle.stats.total_hugs, bundle.user.gender.as_deref()),
        "captcha_type": bundle.user.captcha_type,
    });
    if let Some(c) = bundle.user.captcha_cooldown_until {
        profile["captcha_cooldown_until"] = serde_json::json!(c);
    }
    if let Some(b) = bundle.balance {
        profile["balance"] = serde_json::json!(b.amount);
    }
    if let Some(m) = bundle.mutual {
        profile["mutual_total"] = serde_json::json!(m.total);
        profile["mutual_given"] = serde_json::json!(m.given);
        profile["mutual_received"] = serde_json::json!(m.received);
    }
    if bundle.is_blocked {
        profile["is_blocked"] = serde_json::json!(true);
    }
    if let Some(intimacy) = bundle.intimacy {
        profile["intimacy"] = serde_json::to_value(IntimacyInfoDto::from(&intimacy)).unwrap();
    }
    Ok(Json(profile))
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

pub async fn search_users(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SearchQuery>,
    auth: AuthUser,
) -> HttpResult<Json<Vec<UserListItemDto>>> {
    let query = q.q.unwrap_or_default();
    let limit = q.limit.filter(|v| *v > 0).unwrap_or(20);
    let offset = q.offset.filter(|v| *v >= 0).unwrap_or(0);
    let users = state
        .hug
        .search_users(&query, auth.user_id, limit, offset)
        .await?;
    Ok(Json(users.iter().map(UserListItemDto::from).collect()))
}

pub async fn pair_intimacy(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(other): Path<Uuid>,
) -> HttpResult<Json<IntimacyInfoDto>> {
    let info = state.hug.pair_intimacy(auth.user_id, other).await?;
    Ok(Json(IntimacyInfoDto::from(&info)))
}

#[derive(Debug, Serialize)]
pub struct PairStreakResp {
    pub streak: StreakInfoDto,
    pub calendar: Vec<StreakCalendarDayDto>,
}

pub async fn pair_streak(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(other): Path<Uuid>,
) -> HttpResult<Json<PairStreakResp>> {
    let streak = state.hug.pair_streak(auth.user_id, other).await?;
    let calendar = state.hug.pair_streak_calendar(auth.user_id, other).await?;
    Ok(Json(PairStreakResp {
        streak: StreakInfoDto::from(&streak),
        calendar: calendar.iter().map(StreakCalendarDayDto::from).collect(),
    }))
}

pub async fn top_streaks(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<Vec<TopStreakEntryDto>>> {
    let entries = state.hug.user_top_streaks(auth.user_id, 3).await?;
    Ok(Json(entries.iter().map(TopStreakEntryDto::from).collect()))
}

pub async fn connections(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(p): Query<Pagination>,
) -> HttpResult<Json<Vec<ConnectionItemDto>>> {
    let (limit, offset) = paginated(&p, 20);
    let conns = state
        .hug
        .user_connections(auth.user_id, limit, offset)
        .await?;
    Ok(Json(conns.iter().map(ConnectionItemDto::from).collect()))
}

pub async fn intimacy_leaderboard(
    State(state): State<Arc<AppState>>,
    Query(p): Query<Pagination>,
    _auth: AuthUser,
) -> HttpResult<Json<Vec<IntimacyLeaderboardEntryDto>>> {
    let (limit, offset) = paginated(&p, 20);
    let entries = state.hug.intimacy_leaderboard(limit, offset).await?;
    Ok(Json(entries.iter().map(IntimacyLeaderboardEntryDto::from).collect()))
}

// ── blocks ────────────────────────────────────────────────────────────

pub async fn block_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(target): Path<Uuid>,
) -> HttpResult<Json<serde_json::Value>> {
    state
        .hug
        .block_user(auth.user_id, target)
        .await
        .map_err(|err| match err {
            AppError::CannotBlockSelf => ApiError::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::CannotHugSelf,
                "Cannot block yourself",
            ),
            other => other.into(),
        })?;
    Ok(Json(serde_json::json!({ "message": "User blocked" })))
}

pub async fn unblock_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(target): Path<Uuid>,
) -> HttpResult<Json<serde_json::Value>> {
    state.hug.unblock_user(auth.user_id, target).await?;
    Ok(Json(serde_json::json!({ "message": "User unblocked" })))
}

pub async fn blocked_users(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<Vec<BlockedUserDto>>> {
    let users = state.hug.blocked_users(auth.user_id).await?;
    Ok(Json(users.iter().map(BlockedUserDto::from).collect()))
}

// ── announcements ─────────────────────────────────────────────────────

pub async fn active_announcement(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<axum::response::Response, ApiError> {
    let a = state.user.active_announcement(auth.user_id).await?;
    let Some(a) = a else {
        return Ok((StatusCode::NO_CONTENT, ()).into_response());
    };
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "id": a.id,
            "message": a.message,
            "created_at": a.created_at,
        })),
    )
        .into_response())
}

pub async fn dismiss_announcement(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(announcement_id): Path<Uuid>,
) -> HttpResult<Json<serde_json::Value>> {
    state
        .user
        .dismiss_announcement(auth.user_id, announcement_id)
        .await?;
    Ok(Json(serde_json::json!({ "message": "dismissed" })))
}

use axum::response::IntoResponse;
