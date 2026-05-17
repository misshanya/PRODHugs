use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Concurrency-safe in-memory cache with per-entry TTL.
pub struct TtlCache<K, V> {
    ttl: Duration,
    inner: Mutex<HashMap<K, Entry<V>>>,
}

struct Entry<V> {
    value: V,
    expires_at: Instant,
}

impl<K: Eq + Hash + Clone, V: Clone> TtlCache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let now = Instant::now();
        let guard = self.inner.lock().ok()?;
        guard
            .get(key)
            .filter(|entry| entry.expires_at > now)
            .map(|entry| entry.value.clone())
    }

    pub fn set(&self, key: K, value: V) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.insert(
                key,
                Entry {
                    value,
                    expires_at: Instant::now() + self.ttl,
                },
            );
        }
    }

    pub fn invalidate(&self, key: &K) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.remove(key);
        }
    }

    pub fn invalidate_all(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.clear();
        }
    }
}
