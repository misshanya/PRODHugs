//! Maps `AppError` (and inline `ApiError`) to JSON HTTP responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::error::{AppError, ErrorCode};

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: ErrorCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::BadRequest, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, ErrorCode::Forbidden, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorCode::NotFound, message)
    }

    pub fn conflict(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, message)
    }

    pub fn rate_limited() -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::RateLimited,
            "too many requests, try again later",
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorBody {
            code: self.code,
            message: self.message,
        };
        (self.status, Json(body)).into_response()
    }
}

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        use crate::error::AppError as E;
        let (status, code) = match &err {
            E::UserAlreadyExists => (StatusCode::CONFLICT, ErrorCode::UserAlreadyExists),
            E::UserNotFound => (StatusCode::NOT_FOUND, ErrorCode::UserNotFound),
            E::InvalidCredentials => (StatusCode::UNAUTHORIZED, ErrorCode::InvalidCredentials),
            E::WrongPassword => (StatusCode::BAD_REQUEST, ErrorCode::WrongPassword),
            E::UserBanned => (StatusCode::FORBIDDEN, ErrorCode::UserBanned),
            E::CannotBanAdmin => (StatusCode::BAD_REQUEST, ErrorCode::CannotBanAdmin),
            E::CannotDeleteAdmin => (StatusCode::BAD_REQUEST, ErrorCode::CannotDeleteAdmin),
            E::UserBlocked => (StatusCode::CONFLICT, ErrorCode::UserBlocked),
            E::CannotBlockSelf => (StatusCode::BAD_REQUEST, ErrorCode::CannotHugSelf),
            E::InvalidTelegramId => (StatusCode::BAD_REQUEST, ErrorCode::InvalidTelegramId),
            E::TelegramIdTaken => (StatusCode::CONFLICT, ErrorCode::TelegramIdTaken),
            E::HugCooldownActive => (StatusCode::TOO_MANY_REQUESTS, ErrorCode::CooldownActive),
            E::CannotHugSelf => (StatusCode::BAD_REQUEST, ErrorCode::CannotHugSelf),
            E::InsufficientBalance => (StatusCode::BAD_REQUEST, ErrorCode::InsufficientBalance),
            E::DailyRewardAlreadyClaimed => (StatusCode::CONFLICT, ErrorCode::Conflict),
            E::CooldownNotFound => (StatusCode::NOT_FOUND, ErrorCode::NotFound),
            E::AlreadyHasPendingHug => (StatusCode::CONFLICT, ErrorCode::AlreadyHasPendingHug),
            E::PendingHugExists | E::ReversePendingHugExists => {
                (StatusCode::CONFLICT, ErrorCode::PendingHugExists)
            }
            E::HugNotFound => (StatusCode::NOT_FOUND, ErrorCode::HugNotFound),
            E::HugNotPending => (StatusCode::CONFLICT, ErrorCode::HugNotPending),
            E::HugExpired => (StatusCode::GONE, ErrorCode::HugExpired),
            E::DeclineCooldownActive => {
                (StatusCode::TOO_MANY_REQUESTS, ErrorCode::DeclineCooldownActive)
            }
            E::MaxSlotsReached => (StatusCode::CONFLICT, ErrorCode::MaxSlotsReached),
            E::HugTypeLocked => (StatusCode::CONFLICT, ErrorCode::HugTypeLocked),
            E::CaptchaRequired => (StatusCode::BAD_REQUEST, ErrorCode::CaptchaRequired),
            E::CaptchaFailed => (StatusCode::BAD_REQUEST, ErrorCode::CaptchaFailed),
            E::CaptchaNotFound => (StatusCode::NOT_FOUND, ErrorCode::NotFound),
            E::CaptchaForbidden => (StatusCode::FORBIDDEN, ErrorCode::Forbidden),
            E::CaptchaGone => (StatusCode::GONE, ErrorCode::Gone),
            E::NoteNotFound => (StatusCode::NOT_FOUND, ErrorCode::NoteNotFound),
            E::NoteInvalid => (StatusCode::BAD_REQUEST, ErrorCode::NoteInvalid),
            E::Validation(_) => (StatusCode::BAD_REQUEST, ErrorCode::BadRequest),
            E::Forbidden => (StatusCode::FORBIDDEN, ErrorCode::Forbidden),
            E::Db(_) | E::Migrate(_) | E::Other(_) => {
                tracing::error!(error = ?err, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::Internal)
            }
        };
        ApiError {
            status,
            code,
            message: err.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let api: ApiError = self.into();
        api.into_response()
    }
}

pub type HttpResult<T> = Result<T, ApiError>;
