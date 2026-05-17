//! `/api/v2` routes — username-aware lookups, note CRUD, daily-reward status.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, ErrorCode};
use crate::http::auth::AuthUser;
use crate::http::dto::{IntimacyInfoDto, UserListItemDto, UserNoteDto};
use crate::http::error::{ApiError, HttpResult};
use crate::http::AppState;
use crate::models;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let _ = state;
    Router::new()
        .route("/users/search", get(search_users_v2))
        .route("/users/:username_or_id/profile", get(profile_v2))
        .route("/users/:username_or_id/note", get(get_note))
        .route("/users/:username_or_id/note", put(upsert_note))
        .route("/users/:username_or_id/note", delete(delete_note))
        .route("/notes", get(list_notes))
        .route("/daily-reward/status", get(daily_status))
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

pub async fn search_users_v2(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(q): Query<SearchQuery>,
) -> HttpResult<Json<Vec<UserListItemDto>>> {
    let raw = q.q.unwrap_or_default();
    let (query, username_only) = if let Some(rest) = raw.strip_prefix('@') {
        (rest.to_string(), true)
    } else {
        (raw, false)
    };
    let limit = q.limit.filter(|v| *v > 0).unwrap_or(20);
    let offset = q.offset.filter(|v| *v >= 0).unwrap_or(0);

    let users = state
        .hug
        .search_users(&query, auth.user_id, limit, offset)
        .await?;
    let users = if username_only && !query.is_empty() {
        let needle = query.to_lowercase();
        users
            .into_iter()
            .filter(|u| u.username.to_lowercase().contains(&needle))
            .collect()
    } else {
        users
    };
    Ok(Json(users.iter().map(UserListItemDto::from).collect()))
}

async fn resolve_target(state: &AppState, raw: &str) -> Result<models::User, AppError> {
    if let Ok(id) = Uuid::parse_str(raw) {
        return state.user.get_by_id(id).await;
    }
    let username = raw.strip_prefix('@').unwrap_or(raw);
    if username.is_empty() {
        return Err(AppError::UserNotFound);
    }
    state.user.get_by_username(username).await
}

pub async fn profile_v2(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(raw): Path<String>,
) -> HttpResult<Json<serde_json::Value>> {
    let target = resolve_target(&state, &raw).await?;
    let bundle = state
        .hug
        .get_user_profile(target.id, Some(auth.user_id))
        .await?;
    let mut resp = serde_json::json!({
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
        resp["captcha_cooldown_until"] = serde_json::json!(c);
    }
    if let Some(b) = bundle.balance {
        resp["balance"] = serde_json::json!(b.amount);
    }
    if let Some(m) = bundle.mutual {
        resp["mutual_total"] = serde_json::json!(m.total);
        resp["mutual_given"] = serde_json::json!(m.given);
        resp["mutual_received"] = serde_json::json!(m.received);
    }
    if auth.user_id != target.id {
        resp["is_blocked"] = serde_json::json!(bundle.is_blocked);
    }
    if let Some(i) = bundle.intimacy {
        resp["intimacy"] = serde_json::to_value(IntimacyInfoDto::from(&i)).unwrap();
    }
    Ok(Json(resp))
}

pub async fn get_note(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(raw): Path<String>,
) -> HttpResult<Json<UserNoteDto>> {
    let target = state.note.resolve_target(&raw).await?;
    let note = state.note.get(auth.user_id, target.id).await?.ok_or_else(|| {
        ApiError::new(
            StatusCode::NOT_FOUND,
            ErrorCode::NoteNotFound,
            "Note not found",
        )
    })?;
    Ok(Json(UserNoteDto::from(&note)))
}

#[derive(Debug, Deserialize)]
pub struct UpsertNoteReq {
    pub content: String,
}

pub async fn upsert_note(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(raw): Path<String>,
    Json(req): Json<UpsertNoteReq>,
) -> HttpResult<Json<UserNoteDto>> {
    let target = state.note.resolve_target(&raw).await?;
    let note = state
        .note
        .upsert(auth.user_id, target.id, &req.content)
        .await?;
    Ok(Json(UserNoteDto::from(&note)))
}

pub async fn delete_note(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(raw): Path<String>,
) -> Result<Response, ApiError> {
    let target = state.note.resolve_target(&raw).await?;
    state.note.delete(auth.user_id, target.id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, Deserialize)]
pub struct ListNotesQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

pub async fn list_notes(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(q): Query<ListNotesQuery>,
) -> HttpResult<Json<Vec<UserNoteDto>>> {
    let limit = q.limit.filter(|v| *v > 0).unwrap_or(50);
    let offset = q.offset.filter(|v| *v >= 0).unwrap_or(0);
    let notes = state.note.list(auth.user_id, limit, offset).await?;
    Ok(Json(notes.iter().map(|n| UserNoteDto::from(n)).collect()))
}

#[derive(Debug, Serialize)]
pub struct DailyStatusResp {
    pub can_claim: bool,
    pub next_claim_at: DateTime<Utc>,
    pub streak_days: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_claimed_at: Option<DateTime<Utc>>,
}

pub async fn daily_status(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<DailyStatusResp>> {
    let (can_claim, next_claim_at, streak_days, last_claimed_at) =
        state.hug.daily_reward_status(auth.user_id).await?;
    Ok(Json(DailyStatusResp {
        can_claim,
        next_claim_at,
        streak_days,
        last_claimed_at,
    }))
}
