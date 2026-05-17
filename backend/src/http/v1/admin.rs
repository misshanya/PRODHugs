//! Admin-only endpoints.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::http::auth::AdminAuth;
use crate::http::dto::AdminUserDto;
use crate::http::error::{ApiError, HttpResult};
use crate::http::AppState;

pub async fn ping() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "secret": "PONG_ADMIN" }))
}

pub async fn stats(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
) -> HttpResult<Json<serde_json::Value>> {
    let s = state.user.admin_stats().await?;
    Ok(Json(serde_json::json!({
        "total_users": s.total_users,
        "banned_users": s.banned_users,
    })))
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub q: Option<String>,
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListUsersQuery>,
    _admin: AdminAuth,
) -> HttpResult<Json<Vec<AdminUserDto>>> {
    let limit = q.limit.filter(|v| *v > 0).unwrap_or(20);
    let offset = q.offset.filter(|v| *v >= 0).unwrap_or(0);
    let users = if let Some(query) = q.q.as_deref().filter(|s| !s.is_empty()) {
        state.user.search_admin(query, limit, offset).await?
    } else {
        state.user.list_admin(limit, offset).await?
    };
    Ok(Json(users.iter().map(AdminUserDto::from).collect()))
}

pub async fn ban_user(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
) -> HttpResult<Json<AdminUserDto>> {
    let u = state.user.ban(user_id).await?;
    Ok(Json(AdminUserDto::from(&u)))
}

pub async fn unban_user(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
) -> HttpResult<Json<AdminUserDto>> {
    let u = state.user.unban(user_id).await?;
    Ok(Json(AdminUserDto::from(&u)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateUsernameReq {
    pub username: String,
}

pub async fn update_username(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUsernameReq>,
) -> HttpResult<Json<AdminUserDto>> {
    let u = state.user.admin_update_username(user_id, &req.username).await?;
    Ok(Json(AdminUserDto::from(&u)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateGenderReq {
    pub gender: Option<String>,
}

pub async fn update_gender(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateGenderReq>,
) -> HttpResult<Json<AdminUserDto>> {
    let u = state.user.admin_update_gender(user_id, req.gender.as_deref()).await?;
    Ok(Json(AdminUserDto::from(&u)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateDisplayNameReq {
    pub display_name: Option<String>,
}

pub async fn update_display_name(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateDisplayNameReq>,
) -> HttpResult<Json<AdminUserDto>> {
    let u = state
        .user
        .admin_update_display_name(user_id, req.display_name.as_deref())
        .await?;
    Ok(Json(AdminUserDto::from(&u)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateTagReq {
    pub tag: Option<String>,
}

pub async fn update_tag(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateTagReq>,
) -> HttpResult<Json<AdminUserDto>> {
    let u = state.user.admin_update_tag(user_id, req.tag.as_deref()).await?;
    Ok(Json(AdminUserDto::from(&u)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSpecialTagReq {
    pub special_tag: Option<String>,
}

pub async fn update_special_tag(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateSpecialTagReq>,
) -> HttpResult<Json<AdminUserDto>> {
    let u = state
        .user
        .admin_update_special_tag(user_id, req.special_tag.as_deref())
        .await?;
    Ok(Json(AdminUserDto::from(&u)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateCaptchaTypeReq {
    pub captcha_type: String,
}

pub async fn update_captcha_type(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateCaptchaTypeReq>,
) -> HttpResult<Json<AdminUserDto>> {
    let u = state
        .user
        .admin_update_captcha_type(user_id, &req.captcha_type)
        .await?;
    Ok(Json(AdminUserDto::from(&u)))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordReq {
    pub password: String,
}

pub async fn update_password(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdatePasswordReq>,
) -> HttpResult<Json<serde_json::Value>> {
    state.user.admin_update_password(user_id, &req.password).await?;
    Ok(Json(serde_json::json!({ "message": "password updated" })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateBalanceReq {
    pub amount: i32,
}

pub async fn update_balance(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateBalanceReq>,
) -> HttpResult<Json<serde_json::Value>> {
    let bal = state.user.admin_update_balance(user_id, req.amount).await?;
    Ok(Json(serde_json::json!({
        "user_id": bal.user_id,
        "amount": bal.amount,
    })))
}

pub async fn clear_promotion(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
) -> HttpResult<Json<AdminUserDto>> {
    let u = state.user.admin_clear_promotion(user_id).await?;
    Ok(Json(AdminUserDto::from(&u)))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(user_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    state.user.admin_delete(user_id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

// ── announcements ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateAnnouncementReq {
    pub message: String,
}

pub async fn create_announcement(
    State(state): State<Arc<AppState>>,
    admin: AdminAuth,
    Json(req): Json<CreateAnnouncementReq>,
) -> HttpResult<Json<serde_json::Value>> {
    let a = state
        .user
        .create_announcement(admin.0.user_id, &req.message)
        .await?;
    Ok(Json(serde_json::json!({
        "id": a.id,
        "message": a.message,
        "created_at": a.created_at,
    })))
}

pub async fn delete_announcement(
    State(state): State<Arc<AppState>>,
    _admin: AdminAuth,
    Path(announcement_id): Path<Uuid>,
) -> HttpResult<Json<serde_json::Value>> {
    state.user.deactivate_announcement(announcement_id).await?;
    Ok(Json(serde_json::json!({ "message": "Announcement removed" })))
}
