//! `/api/v1` routes — direct port of the Go handlers.

mod auth;
mod hug;
mod admin;

use std::sync::Arc;
use std::time::Duration;

use axum::routing::{delete, get, patch, post};
use axum::{middleware, Router};

use crate::http::ratelimit::{LimitConfig, LimitTable, Limiter};
use crate::http::AppState;

pub fn router(_state: Arc<AppState>) -> Router<Arc<AppState>> {
    let auth_limiter = Arc::new(Limiter::new(LimitTable {
        entries: {
            let mut m = std::collections::HashMap::new();
            // 2 tokens / hour (one every 30 minutes), burst of 2.
            m.insert(
                "/api/v1/auth/register".to_string(),
                LimitConfig {
                    tokens_per_second: 1.0 / (30.0 * 60.0),
                    burst: 2,
                    ttl: Duration::from_secs(60 * 60),
                },
            );
            // Auth-shared default: 2 tokens/sec, burst 5.
            for path in ["/api/v1/auth/login", "/api/v1/auth/refresh"] {
                m.insert(
                    path.to_string(),
                    LimitConfig {
                        tokens_per_second: 2.0,
                        burst: 5,
                        ttl: Duration::from_secs(10 * 60),
                    },
                );
            }
            m.insert(
                "/api/v1/auth/check-username".to_string(),
                LimitConfig {
                    tokens_per_second: 5.0,
                    burst: 10,
                    ttl: Duration::from_secs(5 * 60),
                },
            );
            m.insert(
                "/api/v1/auth/telegram/init".to_string(),
                LimitConfig {
                    tokens_per_second: 1.0 / 12.0,
                    burst: 5,
                    ttl: Duration::from_secs(60),
                },
            );
            m.insert(
                "/api/v1/auth/telegram/poll".to_string(),
                LimitConfig {
                    tokens_per_second: 2.0,
                    burst: 5,
                    ttl: Duration::from_secs(5 * 60),
                },
            );
            m
        },
        default: None,
    }));

    Router::new()
        // Health
        .route("/ping", get(auth::ping))
        .route("/admin/ping", get(admin::ping))
        // Auth
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/check-username", get(auth::check_username))
        .route("/auth/telegram/init", post(auth::telegram_init))
        .route("/auth/telegram/poll", post(auth::telegram_poll))
        // User self
        .route("/users/me", get(auth::me))
        .route("/users/me/settings", patch(auth::update_settings))
        .route("/users/me/password", post(auth::change_password))
        .route("/users/me/promote", post(auth::promote_self))
        .route("/users/me/telegram", post(auth::telegram_link))
        .route("/users/me/telegram", delete(auth::telegram_unlink))
        .route("/users/me/inbox", get(hug::inbox))
        .route("/users/me/inbox/count", get(hug::inbox_count))
        .route("/users/me/outgoing", get(hug::outgoing))
        .route("/users/me/slots/buy", post(hug::buy_slot))
        .route("/users/me/blocks", get(hug::blocked_users))
        // Captcha
        .route("/captcha/sudoku", get(auth::sudoku_get))
        .route("/captcha/sudoku/:id/verify", post(auth::sudoku_verify))
        .route("/captcha/sudoku/:id/complete", post(auth::sudoku_complete))
        .route("/captcha/casino", post(auth::casino_get))
        .route("/captcha/casino/:id/spin", post(auth::casino_spin))
        // Profiles
        .route("/users/vips", get(auth::vips))
        .route("/users/search", get(hug::search_users))
        .route("/users/:user_id", get(hug::user_profile))
        .route("/users/:user_id/hug", post(hug::suggest_hug))
        .route("/users/:user_id/cooldown", get(hug::get_cooldown))
        .route("/users/:user_id/cooldown/upgrade", post(hug::upgrade_cooldown))
        .route("/users/:user_id/intimacy", get(hug::pair_intimacy))
        .route("/users/:user_id/streak", get(hug::pair_streak))
        .route("/users/:user_id/block", post(hug::block_user))
        .route("/users/:user_id/block", delete(hug::unblock_user))
        // Hug actions
        .route("/hugs/:hug_id", get(hug::get_detail))
        .route("/hugs/:hug_id/accept", post(hug::accept_hug))
        .route("/hugs/:hug_id/decline", post(hug::decline_hug))
        .route("/hugs/:hug_id/cancel", post(hug::cancel_hug))
        // Feed / leaderboard / activity / streaks
        .route("/hugs", get(hug::feed))
        .route("/hugs/history", get(hug::history))
        .route("/hugs/activity", get(hug::activity))
        .route("/leaderboard", get(hug::leaderboard))
        .route("/streaks/top", get(hug::top_streaks))
        .route("/connections", get(hug::connections))
        .route("/connections/leaderboard", get(hug::intimacy_leaderboard))
        // Balance + daily reward
        .route("/balance", get(hug::balance))
        .route("/daily-reward/claim", post(hug::claim_daily))
        // Announcements
        .route("/announcement", get(hug::active_announcement))
        .route("/announcements/:announcement_id/dismiss", post(hug::dismiss_announcement))
        // Admin
        .route("/admin/stats", get(admin::stats))
        .route("/admin/users", get(admin::list_users))
        .route("/admin/users/:user_id/ban", post(admin::ban_user))
        .route("/admin/users/:user_id/unban", post(admin::unban_user))
        .route("/admin/users/:user_id/username", patch(admin::update_username))
        .route("/admin/users/:user_id/gender", patch(admin::update_gender))
        .route("/admin/users/:user_id/display-name", patch(admin::update_display_name))
        .route("/admin/users/:user_id/tag", patch(admin::update_tag))
        .route("/admin/users/:user_id/special-tag", patch(admin::update_special_tag))
        .route("/admin/users/:user_id/captcha-type", patch(admin::update_captcha_type))
        .route("/admin/users/:user_id/password", patch(admin::update_password))
        .route("/admin/users/:user_id/balance", patch(admin::update_balance))
        .route("/admin/users/:user_id/promotion", delete(admin::clear_promotion))
        .route("/admin/users/:user_id", delete(admin::delete_user))
        .route("/admin/announcements", post(admin::create_announcement))
        .route(
            "/admin/announcements/:announcement_id",
            delete(admin::delete_announcement),
        )
        .layer(middleware::from_fn_with_state(
            auth_limiter,
            crate::http::ratelimit::middleware,
        ))
}
