//! Authentication, profile self-management, and captcha endpoints.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::http::auth::AuthUser;
use crate::http::dto::{AuthResponse, UserDto, UserListItemDto};
use crate::http::error::{ApiError, HttpResult};
use crate::http::AppState;
use crate::models::CreateUser;
use crate::telegram;

// ── public ping ────────────────────────────────────────────────────────

pub async fn ping() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "PONG_PUBLIC" }))
}

// ── auth: register / login / refresh / logout ─────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterReq {
    pub username: String,
    pub password: String,
    pub gender: Option<String>,
}

static HAS_LETTER: Lazy<Regex> = Lazy::new(|| Regex::new(r"[a-zA-Z]").unwrap());
static HAS_DIGIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9]").unwrap());
static HAS_SPECIAL: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^a-zA-Z0-9\s]").unwrap());

fn validate_password(pw: &str) -> Result<(), ApiError> {
    if !HAS_LETTER.is_match(pw) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            crate::error::ErrorCode::WeakPassword,
            "password must contain at least one letter",
        ));
    }
    if !HAS_DIGIT.is_match(pw) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            crate::error::ErrorCode::WeakPassword,
            "password must contain at least one digit",
        ));
    }
    if !HAS_SPECIAL.is_match(pw) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            crate::error::ErrorCode::WeakPassword,
            "password must contain at least one special character",
        ));
    }
    Ok(())
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterReq>,
) -> HttpResult<Response> {
    validate_password(&req.password)?;
    let input = CreateUser {
        username: req.username,
        password: req.password.clone(),
        hashed_password: String::new(),
        role: "user".into(),
        gender: req.gender,
    };
    let (user, access, refresh) = state.user.create(input).await?;
    let cookie = make_refresh_cookie(refresh, state.jwt.refresh_ttl(), state.cfg.jwt.cookie_secure);
    let body = AuthResponse {
        token: access,
        user: UserDto::from(&user),
    };
    let mut resp = (StatusCode::CREATED, Json(body)).into_response();
    resp.headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(resp)
}

#[derive(Debug, Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginReq>,
) -> HttpResult<Response> {
    match state.user.login(&req.username, &req.password).await {
        Ok((user, access, refresh)) => {
            let cookie =
                make_refresh_cookie(refresh, state.jwt.refresh_ttl(), state.cfg.jwt.cookie_secure);
            let body = AuthResponse {
                token: access,
                user: UserDto::from(&user),
            };
            let mut resp = (StatusCode::OK, Json(body)).into_response();
            resp.headers_mut()
                .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
            Ok(resp)
        }
        Err(AppError::UserNotFound | AppError::InvalidCredentials) => Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            crate::error::ErrorCode::InvalidCredentials,
            "invalid username or password",
        )),
        Err(AppError::UserBanned) => Err(ApiError::new(
            StatusCode::FORBIDDEN,
            crate::error::ErrorCode::UserBanned,
            "Ваш аккаунт заблокирован",
        )),
        Err(err) => Err(err.into()),
    }
}

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> HttpResult<Response> {
    let cookie = jar.get("refresh_token").ok_or_else(|| {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            crate::error::ErrorCode::InvalidCredentials,
            "missing refresh token",
        )
    })?;

    let parsed = state.jwt.parse_token(cookie.value()).map_err(|_| {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            crate::error::ErrorCode::InvalidCredentials,
            "invalid refresh token",
        )
    })?;
    if parsed.kind != crate::jwt::TokenType::Refresh {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            crate::error::ErrorCode::InvalidCredentials,
            "invalid token type",
        ));
    }
    let jti = parsed.jti.ok_or_else(|| {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            crate::error::ErrorCode::InvalidCredentials,
            "invalid refresh token",
        )
    })?;
    let active = state.user.is_refresh_token_active(&jti).await?;
    if !active {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            crate::error::ErrorCode::InvalidCredentials,
            "refresh token revoked",
        ));
    }
    let user = state.user.get_by_id(parsed.user_id).await.map_err(|e| {
        if matches!(e, AppError::UserNotFound) {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                crate::error::ErrorCode::InvalidCredentials,
                "user not found",
            )
        } else {
            e.into()
        }
    })?;
    if user.banned_at.is_some() {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            crate::error::ErrorCode::UserBanned,
            "Ваш аккаунт заблокирован",
        ));
    }

    let (access, _) = state
        .jwt
        .generate_access_token(user.id, &user.role)
        .map_err(|e| ApiError::from(AppError::Other(e)))?;
    let (new_refresh, new_jti, new_exp) = state
        .jwt
        .generate_refresh_token(user.id)
        .map_err(|e| ApiError::from(AppError::Other(e)))?;
    state.user.revoke_refresh_token(&jti).await?;
    state
        .user
        .save_refresh_token(&new_jti, user.id, new_exp)
        .await?;

    let cookie =
        make_refresh_cookie(new_refresh, state.jwt.refresh_ttl(), state.cfg.jwt.cookie_secure);
    let mut resp = (
        StatusCode::OK,
        Json(serde_json::json!({ "token": access })),
    )
        .into_response();
    resp.headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(resp)
}

pub async fn logout(State(state): State<Arc<AppState>>, jar: CookieJar) -> Response {
    if let Some(c) = jar.get("refresh_token") {
        if let Ok(parsed) = state.jwt.parse_token(c.value()) {
            if let Some(jti) = parsed.jti {
                let _ = state.user.revoke_refresh_token(&jti).await;
            }
        }
    }
    let cookie = expired_refresh_cookie(state.cfg.jwt.cookie_secure);
    let mut resp = (
        StatusCode::OK,
        Json(serde_json::json!({ "message": "logged out" })),
    )
        .into_response();
    resp.headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    resp
}

// ── check username ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CheckUsernameQuery {
    pub username: String,
}

pub async fn check_username(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CheckUsernameQuery>,
) -> HttpResult<Json<serde_json::Value>> {
    match state.user.get_by_username(&q.username).await {
        Ok(_) => Ok(Json(serde_json::json!({ "available": false }))),
        Err(AppError::UserNotFound) => Ok(Json(serde_json::json!({ "available": true }))),
        Err(err) => Err(err.into()),
    }
}

// ── me ─────────────────────────────────────────────────────────────────

pub async fn me(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<UserDto>> {
    let user = state.user.get_by_id(auth.user_id).await?;
    Ok(Json(UserDto::from(&user)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsReq {
    pub gender: Option<String>,
    pub display_name: Option<String>,
    pub tag: Option<String>,
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<UpdateSettingsReq>,
) -> HttpResult<Json<UserDto>> {
    let display_name = req.display_name.as_deref().map(|s| s.trim());
    let display_name = display_name.filter(|s| !s.is_empty());
    let tag = req.tag.as_deref().map(|s| s.trim());
    let tag = tag.filter(|s| !s.is_empty());

    let user = state
        .user
        .update_settings(auth.user_id, req.gender.as_deref(), display_name, tag)
        .await
        .map_err(|err| match err {
            AppError::InsufficientBalance => ApiError::new(
                StatusCode::BAD_REQUEST,
                crate::error::ErrorCode::InsufficientBalance,
                "Недостаточно обнимань для смены тега (нужно 5)",
            ),
            other => other.into(),
        })?;
    Ok(Json(UserDto::from(&user)))
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordReq {
    pub old_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<ChangePasswordReq>,
) -> HttpResult<Json<serde_json::Value>> {
    validate_password(&req.new_password)?;
    state
        .user
        .change_password(auth.user_id, &req.old_password, &req.new_password)
        .await?;
    Ok(Json(serde_json::json!({ "message": "password changed successfully" })))
}

#[derive(Debug, Deserialize)]
pub struct PromoteReq {
    pub bid: i32,
    pub message: Option<String>,
}

pub async fn promote_self(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<PromoteReq>,
) -> HttpResult<Json<UserListItemDto>> {
    let user = state
        .user
        .promote_user(auth.user_id, req.bid, req.message.as_deref())
        .await?;
    Ok(Json(UserListItemDto::from(&user)))
}

pub async fn vips(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> HttpResult<Json<Vec<UserListItemDto>>> {
    let vips = state.user.list_vip().await?;
    Ok(Json(vips.iter().map(UserListItemDto::from).collect()))
}

// ── telegram link / unlink ────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TelegramLinkResp {
    pub token: String,
    pub bot_url: String,
}

pub async fn telegram_link(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<TelegramLinkResp>> {
    let (token, bot_url) = state.user.generate_link_token(auth.user_id)?;
    Ok(Json(TelegramLinkResp { token, bot_url }))
}

pub async fn telegram_unlink(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<UserDto>> {
    let user = state.user.unlink_telegram(auth.user_id).await?;
    Ok(Json(UserDto::from(&user)))
}

// ── telegram login (init / poll) ──────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TelegramInitResp {
    pub bot_url: String,
    pub poll_token: String,
}

pub async fn telegram_init(
    State(state): State<Arc<AppState>>,
) -> HttpResult<Json<TelegramInitResp>> {
    if state.cfg.telegram.bot_username.is_empty() {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            crate::error::ErrorCode::TelegramLoginFailed,
            "Telegram login is not configured",
        ));
    }
    let (bot_token, poll_token) = state.login_store.create_session();
    let bot_url = telegram::deep_link_url(
        &state.cfg.telegram.bot_username,
        &format!("login_{bot_token}"),
    );
    Ok(Json(TelegramInitResp { bot_url, poll_token }))
}

#[derive(Debug, Deserialize)]
pub struct TelegramPollReq {
    pub poll_token: String,
}

pub async fn telegram_poll(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TelegramPollReq>,
) -> Response {
    let Some(result) = state.login_store.poll(&req.poll_token) else {
        return ApiError::new(
            StatusCode::NOT_FOUND,
            crate::error::ErrorCode::TelegramLoginFailed,
            "Login session not found or expired",
        )
        .into_response();
    };
    match result.status {
        telegram::LoginSessionStatus::Pending => {
            (StatusCode::ACCEPTED, Json(serde_json::json!({"status": "pending"}))).into_response()
        }
        telegram::LoginSessionStatus::Authenticated => {
            // Look up the user; generate fresh tokens.
            let user = match state.user.get_by_id(result.user_id).await {
                Ok(u) => u,
                Err(err) => return ApiError::from(err).into_response(),
            };
            let (access, _) = match state.jwt.generate_access_token(user.id, &user.role) {
                Ok(v) => v,
                Err(e) => return ApiError::from(AppError::Other(e)).into_response(),
            };
            let (refresh, jti, exp) = match state.jwt.generate_refresh_token(user.id) {
                Ok(v) => v,
                Err(e) => return ApiError::from(AppError::Other(e)).into_response(),
            };
            if let Err(err) = state.user.save_refresh_token(&jti, user.id, exp).await {
                return ApiError::from(err).into_response();
            }
            let body = AuthResponse {
                token: access,
                user: UserDto::from(&user),
            };
            let cookie =
                make_refresh_cookie(refresh, state.jwt.refresh_ttl(), state.cfg.jwt.cookie_secure);
            let mut resp = (StatusCode::OK, Json(body)).into_response();
            resp.headers_mut()
                .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
            resp
        }
        telegram::LoginSessionStatus::Failed => ApiError::new(
            StatusCode::FORBIDDEN,
            crate::error::ErrorCode::TelegramLoginFailed,
            result.fail_reason,
        )
        .into_response(),
    }
}

// ── captcha ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SudokuCaptchaResp {
    pub id: Uuid,
    pub puzzle: Vec<Vec<i32>>,
}

pub async fn sudoku_get(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<SudokuCaptchaResp>> {
    let (id, puzzle) = state.user.generate_sudoku_captcha(auth.user_id).await?;
    Ok(Json(SudokuCaptchaResp { id, puzzle }))
}

#[derive(Debug, Deserialize)]
pub struct SudokuVerifyReq {
    pub row: i32,
    pub col: i32,
    pub value: i32,
}

#[derive(Debug, Serialize)]
pub struct SudokuVerifyResp {
    pub correct: bool,
    pub errors: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<bool>,
}

pub async fn sudoku_verify(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<SudokuVerifyReq>,
) -> HttpResult<Json<SudokuVerifyResp>> {
    let r = state
        .user
        .verify_sudoku_cell(id, auth.user_id, req.row, req.col, req.value)
        .await?;
    Ok(Json(SudokuVerifyResp {
        correct: r.correct,
        errors: r.errors,
        failed: Some(r.failed),
    }))
}

#[derive(Debug, Serialize)]
pub struct SudokuCompleteResp {
    pub captcha_token: String,
}

pub async fn sudoku_complete(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> HttpResult<Json<SudokuCompleteResp>> {
    let token = state.user.complete_sudoku(id, auth.user_id).await?;
    Ok(Json(SudokuCompleteResp {
        captcha_token: token,
    }))
}

#[derive(Debug, Serialize)]
pub struct CasinoCaptchaResp {
    pub id: Uuid,
    pub expires_at: DateTime<Utc>,
}

pub async fn casino_get(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> HttpResult<Json<CasinoCaptchaResp>> {
    let (id, expires_at) = state.user.generate_casino_captcha(auth.user_id).await?;
    Ok(Json(CasinoCaptchaResp { id, expires_at }))
}

#[derive(Debug, Serialize)]
pub struct CasinoSpinResp {
    pub win: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cooldown_until: Option<DateTime<Utc>>,
}

pub async fn casino_spin(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> HttpResult<Json<CasinoSpinResp>> {
    let r = state.user.spin_casino(id, auth.user_id).await?;
    Ok(Json(CasinoSpinResp {
        win: r.win,
        captcha_token: r.captcha_token,
        cooldown_until: r.cooldown_until,
    }))
}

// ── cookie helpers ────────────────────────────────────────────────────

fn make_refresh_cookie(value: String, ttl: std::time::Duration, secure: bool) -> Cookie<'static> {
    let mut c = Cookie::new("refresh_token", value);
    c.set_path("/api/v1/auth/");
    c.set_http_only(true);
    c.set_secure(secure);
    c.set_same_site(SameSite::Lax);
    c.set_max_age(time::Duration::seconds(ttl.as_secs() as i64));
    c
}

fn expired_refresh_cookie(secure: bool) -> Cookie<'static> {
    let mut c = Cookie::new("refresh_token", "");
    c.set_path("/api/v1/auth/");
    c.set_http_only(true);
    c.set_secure(secure);
    c.set_same_site(SameSite::Lax);
    c.set_max_age(time::Duration::seconds(-1));
    c
}
