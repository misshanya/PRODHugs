use std::fmt;

use thiserror::Error;

/// Stable, machine-readable error codes returned over the wire.
///
/// Kept in sync with the v1 OpenAPI `ErrorCode` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    WeakPassword,
    UserAlreadyExists,
    InvalidCredentials,
    UserNotFound,
    CannotHugSelf,
    CooldownActive,
    InsufficientBalance,
    WrongPassword,
    UserBanned,
    CannotBanAdmin,
    AlreadyHasPendingHug,
    HugNotFound,
    HugNotPending,
    HugExpired,
    DeclineCooldownActive,
    PendingHugExists,
    MaxSlotsReached,
    UserBlocked,
    InvalidTelegramId,
    TelegramIdTaken,
    TelegramLoginFailed,
    HugTypeLocked,
    CannotDeleteAdmin,
    CommentTooLong,
    CaptchaFailed,
    CaptchaRequired,
    NoteNotFound,
    NoteInvalid,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    Gone,
    RateLimited,
    Internal,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::WeakPassword => "WEAK_PASSWORD",
            ErrorCode::UserAlreadyExists => "USER_ALREADY_EXISTS",
            ErrorCode::InvalidCredentials => "INVALID_CREDENTIALS",
            ErrorCode::UserNotFound => "USER_NOT_FOUND",
            ErrorCode::CannotHugSelf => "CANNOT_HUG_SELF",
            ErrorCode::CooldownActive => "COOLDOWN_ACTIVE",
            ErrorCode::InsufficientBalance => "INSUFFICIENT_BALANCE",
            ErrorCode::WrongPassword => "WRONG_PASSWORD",
            ErrorCode::UserBanned => "USER_BANNED",
            ErrorCode::CannotBanAdmin => "CANNOT_BAN_ADMIN",
            ErrorCode::AlreadyHasPendingHug => "ALREADY_HAS_PENDING_HUG",
            ErrorCode::HugNotFound => "HUG_NOT_FOUND",
            ErrorCode::HugNotPending => "HUG_NOT_PENDING",
            ErrorCode::HugExpired => "HUG_EXPIRED",
            ErrorCode::DeclineCooldownActive => "DECLINE_COOLDOWN_ACTIVE",
            ErrorCode::PendingHugExists => "PENDING_HUG_EXISTS",
            ErrorCode::MaxSlotsReached => "MAX_SLOTS_REACHED",
            ErrorCode::UserBlocked => "USER_BLOCKED",
            ErrorCode::InvalidTelegramId => "INVALID_TELEGRAM_ID",
            ErrorCode::TelegramIdTaken => "TELEGRAM_ID_TAKEN",
            ErrorCode::TelegramLoginFailed => "TELEGRAM_LOGIN_FAILED",
            ErrorCode::HugTypeLocked => "HUG_TYPE_LOCKED",
            ErrorCode::CannotDeleteAdmin => "CANNOT_DELETE_ADMIN",
            ErrorCode::CommentTooLong => "COMMENT_TOO_LONG",
            ErrorCode::CaptchaFailed => "CAPTCHA_FAILED",
            ErrorCode::CaptchaRequired => "CAPTCHA_REQUIRED",
            ErrorCode::NoteNotFound => "NOTE_NOT_FOUND",
            ErrorCode::NoteInvalid => "NOTE_INVALID",
            ErrorCode::BadRequest => "BAD_REQUEST",
            ErrorCode::Unauthorized => "UNAUTHORIZED",
            ErrorCode::Forbidden => "FORBIDDEN",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::Conflict => "CONFLICT",
            ErrorCode::Gone => "GONE",
            ErrorCode::RateLimited => "RATE_LIMITED",
            ErrorCode::Internal => "INTERNAL_ERROR",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl serde::Serialize for ErrorCode {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(self.as_str())
    }
}

/// Domain-level errors used by the service and repository layers.
#[derive(Debug, Error)]
pub enum AppError {
    // ─ users
    #[error("user already exists")]
    UserAlreadyExists,
    #[error("user not found")]
    UserNotFound,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("wrong password")]
    WrongPassword,
    #[error("user is banned")]
    UserBanned,
    #[error("cannot ban admin")]
    CannotBanAdmin,
    #[error("cannot delete admin")]
    CannotDeleteAdmin,
    #[error("user is blocked")]
    UserBlocked,
    #[error("cannot block yourself")]
    CannotBlockSelf,
    #[error("invalid telegram ID")]
    InvalidTelegramId,
    #[error("telegram ID already linked to another account")]
    TelegramIdTaken,

    // ─ hugs
    #[error("hug cooldown is still active")]
    HugCooldownActive,
    #[error("cannot hug yourself")]
    CannotHugSelf,
    #[error("insufficient balance")]
    InsufficientBalance,
    #[error("daily reward already claimed today")]
    DailyRewardAlreadyClaimed,
    #[error("cooldown not found for this pair")]
    CooldownNotFound,
    #[error("already has a pending hug")]
    AlreadyHasPendingHug,
    #[error("pending hug already exists for this pair")]
    PendingHugExists,
    #[error("user has already suggested a hug to you")]
    ReversePendingHugExists,
    #[error("hug not found")]
    HugNotFound,
    #[error("hug is not in pending state")]
    HugNotPending,
    #[error("hug suggestion has expired")]
    HugExpired,
    #[error("decline cooldown is active")]
    DeclineCooldownActive,
    #[error("maximum hug slots reached")]
    MaxSlotsReached,
    #[error("hug type not unlocked for this pair")]
    HugTypeLocked,
    #[error("captcha required")]
    CaptchaRequired,
    #[error("captcha failed")]
    CaptchaFailed,
    #[error("captcha not found")]
    CaptchaNotFound,
    #[error("captcha forbidden")]
    CaptchaForbidden,
    #[error("captcha gone")]
    CaptchaGone,

    // ─ notes
    #[error("note not found")]
    NoteNotFound,
    #[error("note is empty or longer than 256 characters")]
    NoteInvalid,

    // ─ misc
    #[error("validation: {0}")]
    Validation(String),
    #[error("forbidden")]
    Forbidden,

    // ─ infrastructure
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    /// Map this domain error to a stable `ErrorCode` for the wire response.
    pub fn code(&self) -> ErrorCode {
        match self {
            AppError::UserAlreadyExists => ErrorCode::UserAlreadyExists,
            AppError::UserNotFound => ErrorCode::UserNotFound,
            AppError::InvalidCredentials => ErrorCode::InvalidCredentials,
            AppError::WrongPassword => ErrorCode::WrongPassword,
            AppError::UserBanned => ErrorCode::UserBanned,
            AppError::CannotBanAdmin => ErrorCode::CannotBanAdmin,
            AppError::CannotDeleteAdmin => ErrorCode::CannotDeleteAdmin,
            AppError::UserBlocked => ErrorCode::UserBlocked,
            AppError::CannotBlockSelf => ErrorCode::CannotHugSelf,
            AppError::InvalidTelegramId => ErrorCode::InvalidTelegramId,
            AppError::TelegramIdTaken => ErrorCode::TelegramIdTaken,
            AppError::HugCooldownActive => ErrorCode::CooldownActive,
            AppError::CannotHugSelf => ErrorCode::CannotHugSelf,
            AppError::InsufficientBalance => ErrorCode::InsufficientBalance,
            AppError::DailyRewardAlreadyClaimed => ErrorCode::Conflict,
            AppError::CooldownNotFound => ErrorCode::NotFound,
            AppError::AlreadyHasPendingHug => ErrorCode::AlreadyHasPendingHug,
            AppError::PendingHugExists => ErrorCode::PendingHugExists,
            AppError::ReversePendingHugExists => ErrorCode::PendingHugExists,
            AppError::HugNotFound => ErrorCode::HugNotFound,
            AppError::HugNotPending => ErrorCode::HugNotPending,
            AppError::HugExpired => ErrorCode::HugExpired,
            AppError::DeclineCooldownActive => ErrorCode::DeclineCooldownActive,
            AppError::MaxSlotsReached => ErrorCode::MaxSlotsReached,
            AppError::HugTypeLocked => ErrorCode::HugTypeLocked,
            AppError::CaptchaRequired => ErrorCode::CaptchaRequired,
            AppError::CaptchaFailed => ErrorCode::CaptchaFailed,
            AppError::CaptchaNotFound => ErrorCode::NotFound,
            AppError::CaptchaForbidden => ErrorCode::Forbidden,
            AppError::CaptchaGone => ErrorCode::Gone,
            AppError::NoteNotFound => ErrorCode::NoteNotFound,
            AppError::NoteInvalid => ErrorCode::NoteInvalid,
            AppError::Validation(_) => ErrorCode::BadRequest,
            AppError::Forbidden => ErrorCode::Forbidden,
            AppError::Db(_) | AppError::Migrate(_) | AppError::Other(_) => ErrorCode::Internal,
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
