use std::collections::HashMap;
use std::time::{Duration, Instant};

use base64::Engine;
use parking_lot::Mutex;
use rand::RngCore;
use uuid::Uuid;

const LINK_TTL: Duration = Duration::from_secs(5 * 60);

struct Entry {
    user_id: Uuid,
    expires_at: Instant,
}

#[derive(Default)]
pub struct LinkStore {
    entries: Mutex<HashMap<String, Entry>>,
}

impl LinkStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_token(&self, user_id: Uuid) -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
        self.entries.lock().insert(
            token.clone(),
            Entry {
                user_id,
                expires_at: Instant::now() + LINK_TTL,
            },
        );
        token
    }

    /// Consume the token. Returns `Some(user_id)` if valid and unexpired;
    /// otherwise returns `None`. The entry is always removed on lookup.
    pub fn consume_token(&self, token: &str) -> Option<Uuid> {
        let mut guard = self.entries.lock();
        let entry = guard.remove(token)?;
        if Instant::now() > entry.expires_at {
            None
        } else {
            Some(entry.user_id)
        }
    }
}
