use std::collections::HashMap;
use std::time::{Duration, Instant};

use base64::Engine;
use parking_lot::Mutex;
use rand::RngCore;
use uuid::Uuid;

const LOGIN_TTL: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginSessionStatus {
    Pending,
    Authenticated,
    Failed,
}

#[derive(Debug, Clone)]
pub struct TelegramUserInfo {
    pub telegram_id: i64,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
}

#[allow(dead_code)]
struct Session {
    bot_token: String,
    status: LoginSessionStatus,
    user_id: Uuid,
    fail_reason: String,
    expires_at: Instant,
}

#[derive(Debug, Clone)]
pub struct PollResult {
    pub status: LoginSessionStatus,
    pub user_id: Uuid,
    pub fail_reason: String,
}

#[derive(Default)]
pub struct LoginStore {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    sessions: HashMap<String, Session>,   // poll_token -> session
    bot_index: HashMap<String, String>,   // bot_token -> poll_token
}

impl LoginStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new session and return `(bot_token, poll_token)`.
    pub fn create_session(&self) -> (String, String) {
        let bot_token = random_token();
        let poll_token = random_token();
        let session = Session {
            bot_token: bot_token.clone(),
            status: LoginSessionStatus::Pending,
            user_id: Uuid::nil(),
            fail_reason: String::new(),
            expires_at: Instant::now() + LOGIN_TTL,
        };
        let mut inner = self.inner.lock();
        inner.sessions.insert(poll_token.clone(), session);
        inner.bot_index.insert(bot_token.clone(), poll_token.clone());
        (bot_token, poll_token)
    }

    pub fn consume_bot_token(&self, bot_token: &str) -> Option<String> {
        let mut inner = self.inner.lock();
        let poll_token = inner.bot_index.remove(bot_token)?;
        let session = inner.sessions.get(&poll_token)?;
        if Instant::now() > session.expires_at {
            inner.sessions.remove(&poll_token);
            return None;
        }
        Some(poll_token)
    }

    pub fn authenticate(&self, poll_token: &str, user_id: Uuid) {
        if let Some(s) = self.inner.lock().sessions.get_mut(poll_token) {
            s.status = LoginSessionStatus::Authenticated;
            s.user_id = user_id;
        }
    }

    pub fn fail(&self, poll_token: &str, reason: String) {
        if let Some(s) = self.inner.lock().sessions.get_mut(poll_token) {
            s.status = LoginSessionStatus::Failed;
            s.fail_reason = reason;
        }
    }

    pub fn poll(&self, poll_token: &str) -> Option<PollResult> {
        let mut inner = self.inner.lock();
        let session = inner.sessions.get(poll_token)?;
        if Instant::now() > session.expires_at {
            inner.sessions.remove(poll_token);
            return None;
        }
        let result = PollResult {
            status: session.status,
            user_id: session.user_id,
            fail_reason: session.fail_reason.clone(),
        };
        if matches!(
            session.status,
            LoginSessionStatus::Authenticated | LoginSessionStatus::Failed
        ) {
            inner.sessions.remove(poll_token);
        }
        Some(result)
    }
}

fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}
