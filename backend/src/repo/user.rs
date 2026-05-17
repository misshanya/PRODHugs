use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{AdminStats, AdminUser, CreateUser, User};
use crate::repo::map_unique;

const USER_BY_ID_SQL: &str = r#"
SELECT
    u.id, u.username, u.password, u.role,
    u.gender, u.display_name, u.tag, u.special_tag,
    u.telegram_id, u.banned_at, u.created_at,
    u.captcha_type, u.captcha_cooldown_until,
    u.promoted_until, u.promotion_message, u.promotion_bid,
    u.vip_remaining_seconds, u.vip_cooldown_until,
    COALESCE(b.amount, 0)::int AS balance,
    COALESCE((
        SELECT AVG(EXTRACT(EPOCH FROM (h.accepted_at - h.created_at)))
        FROM (
            SELECT accepted_at, created_at
            FROM hugs
            WHERE receiver_id = u.id AND status = 'completed'
            ORDER BY created_at DESC
            LIMIT 30
        ) h
    ), -1)::float8 AS avg_response_time
FROM users u
LEFT JOIN balances b ON b.user_id = u.id
WHERE u.id = $1
"#;

const USER_BY_USERNAME_SQL: &str = r#"
SELECT
    u.id, u.username, u.password, u.role,
    u.gender, u.display_name, u.tag, u.special_tag,
    u.telegram_id, u.banned_at, u.created_at,
    u.captcha_type, u.captcha_cooldown_until,
    u.promoted_until, u.promotion_message, u.promotion_bid,
    u.vip_remaining_seconds, u.vip_cooldown_until,
    COALESCE(b.amount, 0)::int AS balance,
    COALESCE((
        SELECT AVG(EXTRACT(EPOCH FROM (h.accepted_at - h.created_at)))
        FROM (
            SELECT accepted_at, created_at
            FROM hugs
            WHERE receiver_id = u.id AND status = 'completed'
            ORDER BY created_at DESC
            LIMIT 30
        ) h
    ), -1)::float8 AS avg_response_time
FROM users u
LEFT JOIN balances b ON b.user_id = u.id
WHERE u.username = $1
"#;

const USER_BY_TG_SQL: &str = r#"
SELECT
    u.id, u.username, u.password, u.role,
    u.gender, u.display_name, u.tag, u.special_tag,
    u.telegram_id, u.banned_at, u.created_at,
    u.captcha_type, u.captcha_cooldown_until,
    u.promoted_until, u.promotion_message, u.promotion_bid,
    u.vip_remaining_seconds, u.vip_cooldown_until,
    COALESCE(b.amount, 0)::int AS balance,
    COALESCE((
        SELECT AVG(EXTRACT(EPOCH FROM (h.accepted_at - h.created_at)))
        FROM (
            SELECT accepted_at, created_at
            FROM hugs
            WHERE receiver_id = u.id AND status = 'completed'
            ORDER BY created_at DESC
            LIMIT 30
        ) h
    ), -1)::float8 AS avg_response_time
FROM users u
LEFT JOIN balances b ON b.user_id = u.id
WHERE u.telegram_id = $1
"#;

#[derive(sqlx::FromRow)]
struct FullUserRow {
    id: Uuid,
    username: String,
    password: String,
    role: String,
    gender: Option<String>,
    display_name: Option<String>,
    tag: Option<String>,
    special_tag: Option<String>,
    telegram_id: Option<i64>,
    banned_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    captcha_type: String,
    captcha_cooldown_until: Option<DateTime<Utc>>,
    promoted_until: Option<DateTime<Utc>>,
    promotion_message: Option<String>,
    promotion_bid: i32,
    vip_remaining_seconds: i32,
    vip_cooldown_until: Option<DateTime<Utc>>,
    balance: i32,
    avg_response_time: f64,
}

impl From<FullUserRow> for User {
    fn from(row: FullUserRow) -> Self {
        User {
            id: row.id,
            username: row.username,
            role: row.role,
            hashed_password: row.password,
            gender: row.gender,
            display_name: row.display_name,
            tag: row.tag,
            special_tag: row.special_tag,
            telegram_id: row.telegram_id,
            banned_at: row.banned_at,
            created_at: row.created_at,
            captcha_type: row.captcha_type,
            captcha_cooldown_until: row.captcha_cooldown_until,
            promoted_until: row.promoted_until,
            promotion_message: row.promotion_message,
            promotion_bid: row.promotion_bid,
            vip_remaining_seconds: row.vip_remaining_seconds,
            vip_cooldown_until: row.vip_cooldown_until,
            is_recently_active: false,
            is_telegram_linked: row.telegram_id.is_some(),
            avg_response_time: (row.avg_response_time >= 0.0).then_some(row.avg_response_time),
            balance: row.balance,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BaseUserRow {
    id: Uuid,
    username: String,
    password: String,
    role: String,
    gender: Option<String>,
    display_name: Option<String>,
    tag: Option<String>,
    special_tag: Option<String>,
    telegram_id: Option<i64>,
    banned_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    captcha_type: String,
    captcha_cooldown_until: Option<DateTime<Utc>>,
    promoted_until: Option<DateTime<Utc>>,
    promotion_message: Option<String>,
    promotion_bid: i32,
    vip_remaining_seconds: i32,
    vip_cooldown_until: Option<DateTime<Utc>>,
}

impl From<BaseUserRow> for User {
    fn from(r: BaseUserRow) -> Self {
        User {
            id: r.id,
            username: r.username,
            role: r.role,
            hashed_password: r.password,
            gender: r.gender,
            display_name: r.display_name,
            tag: r.tag,
            special_tag: r.special_tag,
            telegram_id: r.telegram_id,
            banned_at: r.banned_at,
            created_at: r.created_at,
            captcha_type: r.captcha_type,
            captcha_cooldown_until: r.captcha_cooldown_until,
            promoted_until: r.promoted_until,
            promotion_message: r.promotion_message,
            promotion_bid: r.promotion_bid,
            vip_remaining_seconds: r.vip_remaining_seconds,
            vip_cooldown_until: r.vip_cooldown_until,
            is_recently_active: false,
            is_telegram_linked: r.telegram_id.is_some(),
            avg_response_time: None,
            balance: 0,
        }
    }
}

pub async fn create<'e, E>(exec: E, input: &CreateUser) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "INSERT INTO users (username, password, role, gender, created_at) \
         VALUES ($1, $2, $3, $4, NOW()) RETURNING *",
    )
    .bind(&input.username)
    .bind(&input.hashed_password)
    .bind(&input.role)
    .bind(input.gender.as_deref())
    .fetch_one(exec)
    .await
    .map_err(|e| map_unique(e, AppError::UserAlreadyExists))?;
    Ok(row.into())
}

pub async fn get_by_id<'e, E>(exec: E, id: Uuid) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as::<_, FullUserRow>(USER_BY_ID_SQL)
        .bind(id)
        .fetch_optional(exec)
        .await?
        .map(Into::into)
        .ok_or(AppError::UserNotFound)
}

pub async fn get_by_username<'e, E>(exec: E, username: &str) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as::<_, FullUserRow>(USER_BY_USERNAME_SQL)
        .bind(username)
        .fetch_optional(exec)
        .await?
        .map(Into::into)
        .ok_or(AppError::UserNotFound)
}

pub async fn get_by_telegram_id<'e, E>(exec: E, telegram_id: i64) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as::<_, FullUserRow>(USER_BY_TG_SQL)
        .bind(telegram_id)
        .fetch_optional(exec)
        .await?
        .map(Into::into)
        .ok_or(AppError::UserNotFound)
}

pub async fn update_settings<'e, E>(
    exec: E,
    id: Uuid,
    gender: Option<&str>,
    display_name: Option<&str>,
    tag: Option<&str>,
) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "UPDATE users SET gender = $2, display_name = $3, tag = $4 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(gender)
    .bind(display_name)
    .bind(tag)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn get_telegram_id<'e, E>(exec: E, user_id: Uuid) -> AppResult<Option<i64>>
where
    E: PgExecutor<'e>,
{
    let (tid,): (Option<i64>,) = sqlx::query_as("SELECT telegram_id FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(exec)
        .await?;
    Ok(tid)
}

pub async fn set_telegram_id<'e, E>(exec: E, user_id: Uuid, telegram_id: i64) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "UPDATE users SET telegram_id = $2 WHERE id = $1 RETURNING *",
    )
    .bind(user_id)
    .bind(telegram_id)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn clear_telegram_id<'e, E>(exec: E, user_id: Uuid) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "UPDATE users SET telegram_id = NULL WHERE id = $1 RETURNING *",
    )
    .bind(user_id)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn is_telegram_id_taken<'e, E>(
    exec: E,
    telegram_id: i64,
    exclude_user_id: Uuid,
) -> AppResult<bool>
where
    E: PgExecutor<'e>,
{
    let (taken,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM users WHERE telegram_id = $1 AND id != $2) AS taken",
    )
    .bind(telegram_id)
    .bind(exclude_user_id)
    .fetch_one(exec)
    .await?;
    Ok(taken)
}

pub async fn update_password<'e, E>(exec: E, id: Uuid, hashed: &str) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("UPDATE users SET password = $2 WHERE id = $1")
        .bind(id)
        .bind(hashed)
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn ban<'e, E>(exec: E, id: Uuid) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: Option<BaseUserRow> = sqlx::query_as(
        "UPDATE users SET banned_at = NOW() WHERE id = $1 AND role != 'admin' RETURNING *",
    )
    .bind(id)
    .fetch_optional(exec)
    .await?;
    row.map(Into::into).ok_or(AppError::CannotBanAdmin)
}

pub async fn unban<'e, E>(exec: E, id: Uuid) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: Option<BaseUserRow> = sqlx::query_as(
        "UPDATE users SET banned_at = NULL WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .fetch_optional(exec)
    .await?;
    row.map(Into::into).ok_or(AppError::UserNotFound)
}

pub async fn count_users<'e, E>(exec: E) -> AppResult<i64>
where
    E: PgExecutor<'e>,
{
    let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(exec)
        .await?;
    Ok(c)
}

pub async fn count_banned_users<'e, E>(exec: E) -> AppResult<i64>
where
    E: PgExecutor<'e>,
{
    let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE banned_at IS NOT NULL")
        .fetch_one(exec)
        .await?;
    Ok(c)
}

pub async fn admin_stats<'e, E>(exec: E) -> AppResult<AdminStats>
where
    E: PgExecutor<'e> + Copy,
{
    let total = count_users(exec).await?;
    let banned = count_banned_users(exec).await?;
    Ok(AdminStats {
        total_users: total,
        banned_users: banned,
    })
}

#[derive(sqlx::FromRow)]
struct AdminUserRow {
    id: Uuid,
    username: String,
    role: String,
    gender: Option<String>,
    display_name: Option<String>,
    tag: Option<String>,
    special_tag: Option<String>,
    banned_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    captcha_type: String,
    captcha_cooldown_until: Option<DateTime<Utc>>,
    promoted_until: Option<DateTime<Utc>>,
    promotion_message: Option<String>,
    promotion_bid: i32,
    vip_remaining_seconds: i32,
    vip_cooldown_until: Option<DateTime<Utc>>,
    balance: i32,
    last_visit_at: Option<DateTime<Utc>>,
}

impl From<AdminUserRow> for AdminUser {
    fn from(r: AdminUserRow) -> Self {
        AdminUser {
            id: r.id,
            username: r.username,
            role: r.role,
            gender: r.gender,
            display_name: r.display_name,
            tag: r.tag,
            special_tag: r.special_tag,
            banned_at: r.banned_at,
            created_at: r.created_at,
            balance: r.balance,
            last_visit_at: r.last_visit_at,
            captcha_type: r.captcha_type,
            captcha_cooldown_until: r.captcha_cooldown_until,
            promoted_until: r.promoted_until,
            promotion_message: r.promotion_message,
            promotion_bid: r.promotion_bid,
            vip_remaining_seconds: r.vip_remaining_seconds,
            vip_cooldown_until: r.vip_cooldown_until,
        }
    }
}

pub async fn list_admin<'e, E>(exec: E, limit: i32, offset: i32) -> AppResult<Vec<AdminUser>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<AdminUserRow> = sqlx::query_as(
        "SELECT u.id, u.username, u.role, u.gender, u.display_name, u.tag, u.special_tag, \
                u.banned_at, u.created_at, u.captcha_type, u.captcha_cooldown_until, \
                u.promoted_until, u.promotion_message, u.promotion_bid, \
                u.vip_remaining_seconds, u.vip_cooldown_until, \
                COALESCE(b.amount, 0)::int AS balance, \
                COALESCE(rt.last_visit, u.created_at)::timestamptz AS last_visit_at \
         FROM users u \
         LEFT JOIN balances b ON b.user_id = u.id \
         LEFT JOIN LATERAL (SELECT MAX(created_at) AS last_visit FROM refresh_tokens \
                            WHERE user_id = u.id) rt ON true \
         ORDER BY last_visit_at DESC NULLS LAST \
         LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(exec)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn search_admin<'e, E>(
    exec: E,
    query: &str,
    limit: i32,
    offset: i32,
) -> AppResult<Vec<AdminUser>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<AdminUserRow> = sqlx::query_as(
        "SELECT u.id, u.username, u.role, u.gender, u.display_name, u.tag, u.special_tag, \
                u.banned_at, u.created_at, u.captcha_type, u.captcha_cooldown_until, \
                u.promoted_until, u.promotion_message, u.promotion_bid, \
                u.vip_remaining_seconds, u.vip_cooldown_until, \
                COALESCE(b.amount, 0)::int AS balance, \
                COALESCE(rt.last_visit, u.created_at)::timestamptz AS last_visit_at \
         FROM users u \
         LEFT JOIN balances b ON b.user_id = u.id \
         LEFT JOIN LATERAL (SELECT MAX(created_at) AS last_visit FROM refresh_tokens \
                            WHERE user_id = u.id) rt ON true \
         WHERE u.username ILIKE '%' || $1 || '%' OR u.display_name ILIKE '%' || $1 || '%' \
         ORDER BY last_visit_at DESC NULLS LAST \
         LIMIT $2 OFFSET $3",
    )
    .bind(query)
    .bind(limit)
    .bind(offset)
    .fetch_all(exec)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn admin_update_username<'e, E>(exec: E, id: Uuid, username: &str) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "UPDATE users SET username = $2 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(username)
    .fetch_one(exec)
    .await
    .map_err(|e| map_unique(e, AppError::UserAlreadyExists))?;
    Ok(row.into())
}

pub async fn admin_update_gender<'e, E>(exec: E, id: Uuid, gender: Option<&str>) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "UPDATE users SET gender = $2 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(gender)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn admin_update_password<'e, E>(exec: E, id: Uuid, hashed: &str) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("UPDATE users SET password = $2 WHERE id = $1")
        .bind(id)
        .bind(hashed)
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn admin_update_display_name<'e, E>(
    exec: E,
    id: Uuid,
    display_name: Option<&str>,
) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "UPDATE users SET display_name = $2 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(display_name)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn admin_update_tag<'e, E>(exec: E, id: Uuid, tag: Option<&str>) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "UPDATE users SET tag = $2 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(tag)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn admin_update_special_tag<'e, E>(
    exec: E,
    id: Uuid,
    special_tag: Option<&str>,
) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "UPDATE users SET special_tag = $2 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(special_tag)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn admin_update_captcha_type<'e, E>(
    exec: E,
    id: Uuid,
    captcha_type: &str,
) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: Option<BaseUserRow> = sqlx::query_as(
        "UPDATE users SET captcha_type = $2 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(captcha_type)
    .fetch_optional(exec)
    .await?;
    row.map(Into::into).ok_or(AppError::UserNotFound)
}

pub async fn admin_delete<'e, E>(exec: E, id: Uuid) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    let res = sqlx::query("DELETE FROM users WHERE id = $1 AND role != 'admin'")
        .bind(id)
        .execute(exec)
        .await?;
    if res.rows_affected() == 0 {
        Err(AppError::CannotDeleteAdmin)
    } else {
        Ok(())
    }
}

pub async fn clear_expired_promotions<'e, E>(exec: E) -> AppResult<i64>
where
    E: PgExecutor<'e>,
{
    let res = sqlx::query(
        "UPDATE users SET promoted_until = NULL, promotion_message = NULL, promotion_bid = 0 \
         WHERE promoted_until < NOW()",
    )
    .execute(exec)
    .await?;
    Ok(res.rows_affected() as i64)
}

pub async fn admin_clear_promotion<'e, E>(exec: E, id: Uuid) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: Option<BaseUserRow> = sqlx::query_as(
        "UPDATE users SET promoted_until = NULL, promotion_message = NULL, promotion_bid = 0 \
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .fetch_optional(exec)
    .await?;
    row.map(Into::into).ok_or(AppError::UserNotFound)
}

pub async fn promote<'e, E>(
    exec: E,
    id: Uuid,
    promoted_until: DateTime<Utc>,
    message: Option<&str>,
    bid: i32,
) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "UPDATE users SET promoted_until = $2, promotion_message = $3, promotion_bid = $4 \
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(promoted_until)
    .bind(message)
    .bind(bid)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn set_vip_cooldown<'e, E>(
    exec: E,
    id: Uuid,
    cooldown_until: DateTime<Utc>,
    remaining_seconds: i32,
) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "UPDATE users SET vip_cooldown_until = $2, vip_remaining_seconds = $3 \
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(cooldown_until)
    .bind(remaining_seconds)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn update_vip_budget<'e, E>(
    exec: E,
    id: Uuid,
    remaining_seconds: i32,
) -> AppResult<User>
where
    E: PgExecutor<'e>,
{
    let row: BaseUserRow = sqlx::query_as(
        "UPDATE users SET vip_remaining_seconds = $2 WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(remaining_seconds)
    .fetch_one(exec)
    .await?;
    Ok(row.into())
}

pub async fn set_captcha_cooldown<'e, E>(
    exec: E,
    user_id: Uuid,
    until: DateTime<Utc>,
) -> AppResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query("UPDATE users SET captcha_cooldown_until = $2 WHERE id = $1")
        .bind(user_id)
        .bind(until)
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn get_slots<'e, E>(exec: E, user_id: Uuid) -> AppResult<i32>
where
    E: PgExecutor<'e>,
{
    let (s,): (i32,) = sqlx::query_as("SELECT hug_slots FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(exec)
        .await?;
    Ok(s)
}

pub async fn increment_slots<'e, E>(exec: E, user_id: Uuid) -> AppResult<Option<i32>>
where
    E: PgExecutor<'e>,
{
    let row: Option<(i32,)> = sqlx::query_as(
        "UPDATE users SET hug_slots = hug_slots + 1 WHERE id = $1 AND hug_slots < 5 \
         RETURNING hug_slots",
    )
    .bind(user_id)
    .fetch_optional(exec)
    .await?;
    Ok(row.map(|(s,)| s))
}

#[derive(sqlx::FromRow)]
struct VipRow {
    id: Uuid,
    username: String,
    role: String,
    gender: Option<String>,
    display_name: Option<String>,
    tag: Option<String>,
    special_tag: Option<String>,
    is_telegram_linked: bool,
    promoted_until: Option<DateTime<Utc>>,
    promotion_message: Option<String>,
    promotion_bid: i32,
    vip_remaining_seconds: i32,
    vip_cooldown_until: Option<DateTime<Utc>>,
    is_recently_active: bool,
    avg_response_time: f64,
}

pub async fn list_vip<'e, E>(exec: E) -> AppResult<Vec<User>>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<VipRow> = sqlx::query_as(
        "SELECT u.id, u.username, u.role, u.gender, u.display_name, u.tag, u.special_tag, \
                (u.telegram_id IS NOT NULL)::bool AS is_telegram_linked, \
                u.promoted_until, u.promotion_message, u.promotion_bid, \
                u.vip_remaining_seconds, u.vip_cooldown_until, \
                (EXISTS (SELECT 1 FROM hugs WHERE receiver_id = u.id AND status = 'completed' AND accepted_at > NOW() - interval '3 days'))::bool AS is_recently_active, \
                COALESCE((SELECT AVG(EXTRACT(EPOCH FROM (h.accepted_at - h.created_at))) FROM (SELECT accepted_at, created_at FROM hugs WHERE receiver_id = u.id AND status = 'completed' ORDER BY created_at DESC LIMIT 30) h), -1)::float8 AS avg_response_time \
         FROM users u \
         WHERE u.promoted_until > NOW() AND u.banned_at IS NULL \
         ORDER BY u.promotion_bid DESC",
    )
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| User {
            id: r.id,
            username: r.username,
            role: r.role,
            hashed_password: String::new(),
            gender: r.gender,
            display_name: r.display_name,
            tag: r.tag,
            special_tag: r.special_tag,
            telegram_id: None,
            banned_at: None,
            created_at: None,
            captcha_type: String::new(),
            captcha_cooldown_until: None,
            promoted_until: r.promoted_until,
            promotion_message: r.promotion_message,
            promotion_bid: r.promotion_bid,
            vip_remaining_seconds: r.vip_remaining_seconds,
            vip_cooldown_until: r.vip_cooldown_until,
            is_recently_active: r.is_recently_active,
            is_telegram_linked: r.is_telegram_linked,
            avg_response_time: (r.avg_response_time >= 0.0).then_some(r.avg_response_time),
            balance: 0,
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct SearchUserRow {
    id: Uuid,
    username: String,
    role: String,
    gender: Option<String>,
    display_name: Option<String>,
    tag: Option<String>,
    special_tag: Option<String>,
    is_telegram_linked: bool,
    promoted_until: Option<DateTime<Utc>>,
    promotion_message: Option<String>,
    promotion_bid: i32,
    vip_remaining_seconds: i32,
    vip_cooldown_until: Option<DateTime<Utc>>,
    is_recently_active: bool,
    avg_response_time: f64,
}

impl From<SearchUserRow> for User {
    fn from(r: SearchUserRow) -> Self {
        User {
            id: r.id,
            username: r.username,
            role: r.role,
            hashed_password: String::new(),
            gender: r.gender,
            display_name: r.display_name,
            tag: r.tag,
            special_tag: r.special_tag,
            telegram_id: None,
            banned_at: None,
            created_at: None,
            captcha_type: String::new(),
            captcha_cooldown_until: None,
            promoted_until: r.promoted_until,
            promotion_message: r.promotion_message,
            promotion_bid: r.promotion_bid,
            vip_remaining_seconds: r.vip_remaining_seconds,
            vip_cooldown_until: r.vip_cooldown_until,
            is_recently_active: r.is_recently_active,
            is_telegram_linked: r.is_telegram_linked,
            avg_response_time: (r.avg_response_time >= 0.0).then_some(r.avg_response_time),
            balance: 0,
        }
    }
}

pub async fn search_users<'e, E>(
    exec: E,
    query: &str,
    viewer_id: Uuid,
    limit: i32,
    offset: i32,
) -> AppResult<Vec<User>>
where
    E: PgExecutor<'e>,
{
    let common_select = "SELECT u.id, u.username, u.role, u.gender, u.display_name, u.tag, u.special_tag, \
            (u.telegram_id IS NOT NULL)::bool AS is_telegram_linked, \
            u.promoted_until, u.promotion_message, u.promotion_bid, \
            u.vip_remaining_seconds, u.vip_cooldown_until, \
            (EXISTS (SELECT 1 FROM hugs WHERE (receiver_id = u.id OR giver_id = u.id) AND status = 'completed' AND accepted_at > NOW() - interval '3 days'))::bool AS is_recently_active, \
            COALESCE((SELECT AVG(EXTRACT(EPOCH FROM (h.accepted_at - h.created_at))) FROM (SELECT accepted_at, created_at FROM hugs WHERE receiver_id = u.id AND status = 'completed' ORDER BY created_at DESC LIMIT 30) h), -1)::float8 AS avg_response_time \
         FROM users u \
         LEFT JOIN LATERAL (SELECT MAX(created_at) AS last_visit FROM refresh_tokens WHERE user_id = u.id) rt ON true ";

    let order_clause = " ORDER BY \
            (u.promoted_until > NOW()) DESC, \
            u.promotion_bid DESC, \
            (EXISTS (SELECT 1 FROM hugs WHERE (receiver_id = u.id OR giver_id = u.id) AND status = 'completed' AND accepted_at > NOW() - interval '3 days')) DESC, \
            COALESCE(rt.last_visit, u.created_at) DESC NULLS LAST ";

    let filter = " u.banned_at IS NULL \
            AND u.id NOT IN ( \
                SELECT blocked_id FROM user_blocks WHERE blocker_id = $1 \
                UNION SELECT blocker_id FROM user_blocks WHERE blocked_id = $1 \
            ) ";

    let rows: Vec<SearchUserRow> = if query.is_empty() {
        let sql = format!(
            "{}WHERE{}{}LIMIT $2 OFFSET $3",
            common_select, filter, order_clause
        );
        sqlx::query_as(&sql)
            .bind(viewer_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(exec)
            .await?
    } else {
        let sql = format!(
            "{}WHERE (u.username ILIKE '%' || $2 || '%' OR u.display_name ILIKE '%' || $2 || '%') AND{}{}LIMIT $3 OFFSET $4",
            common_select, filter, order_clause
        );
        sqlx::query_as(&sql)
            .bind(viewer_id)
            .bind(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(exec)
            .await?
    };
    Ok(rows.into_iter().map(Into::into).collect())
}
