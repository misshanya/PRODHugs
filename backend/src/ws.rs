//! WebSocket Hub — tracks authenticated connections and supports broadcast /
//! send-to-user payloads. Authentication is performed via the first message
//! the client sends: `{"type":"auth","token":"<JWT>"}`.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use parking_lot::RwLock;
use serde::Serialize;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::time::interval;
use uuid::Uuid;

use crate::jwt::{JwtManager, TokenType};
use crate::metrics::MetricsHandle;

const PING_INTERVAL: Duration = Duration::from_secs(30);
const READ_TIMEOUT: Duration = Duration::from_secs(90);
const AUTH_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Hub {
    jwt: Arc<JwtManager>,
    metrics: Arc<MetricsHandle>,
    state: RwLock<HubState>,
}

#[derive(Default)]
struct HubState {
    next_id: u64,
    clients: HashMap<u64, Client>,
    user_index: HashMap<Uuid, HashSet<u64>>,
}

#[allow(dead_code)]
struct Client {
    user_id: Uuid,
    role: String,
    sender: UnboundedSender<Vec<u8>>,
}

#[derive(Serialize)]
struct Envelope<'a, T: Serialize> {
    #[serde(rename = "type")]
    kind: &'a str,
    data: &'a T,
}

impl Hub {
    pub fn new(jwt: Arc<JwtManager>, metrics: Arc<MetricsHandle>) -> Self {
        Self {
            jwt,
            metrics,
            state: RwLock::new(HubState::default()),
        }
    }

    /// Drive a freshly upgraded WebSocket. Returns when the connection closes
    /// or fails authentication.
    pub async fn handle_socket(self: Arc<Self>, socket: WebSocket) {
        let mut socket = socket;

        // Authenticate within `AUTH_TIMEOUT`.
        let first =
            tokio::time::timeout(AUTH_TIMEOUT, async {
                while let Some(msg) = socket.recv().await {
                    match msg {
                        Ok(Message::Text(text)) => return Some(text.to_string()),
                        Ok(Message::Binary(data)) => match std::str::from_utf8(&data) {
                            Ok(s) => return Some(s.to_owned()),
                            Err(_) => continue,
                        },
                        Ok(Message::Ping(_) | Message::Pong(_)) => continue,
                        Ok(Message::Close(_)) | Err(_) => return None,
                    }
                }
                None
            })
            .await
            .ok()
            .flatten();

        let Some(first) = first else {
            return;
        };

        let parsed: Result<AuthFrame, _> = serde_json::from_str(&first);
        let Ok(auth) = parsed else { return };
        if auth.kind != "auth" || auth.token.is_empty() {
            return;
        }
        let parsed = match self.jwt.parse_token(&auth.token) {
            Ok(p) => p,
            Err(_) => return,
        };
        if parsed.kind != TokenType::Access {
            return;
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let client_id = self.register(parsed.user_id, parsed.role.clone(), tx.clone());
        // Send current state on connect.
        let online_users = self.online_users_payload();
        let _ = tx.send(online_users);
        if parsed.role == "admin" {
            let online_count = self.online_count_payload();
            let _ = tx.send(online_count);
        }

        let (mut sender, mut receiver) = socket.split();

        // Write pump.
        let write_task = tokio::spawn(async move {
            let mut ticker = interval(PING_INTERVAL);
            let ping_payload = br#"{"type":"ping"}"#;
            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(bytes) => {
                                if sender.send(Message::Text(String::from_utf8_lossy(&bytes).to_string().into())).await.is_err() {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    _ = ticker.tick() => {
                        if sender.send(Message::Text(String::from_utf8_lossy(ping_payload).to_string().into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        // Read pump — keep the connection alive; close on idle/error.
        loop {
            let msg = tokio::time::timeout(READ_TIMEOUT, receiver.next()).await;
            match msg {
                Ok(Some(Ok(_))) => continue,
                _ => break,
            }
        }

        self.unregister(client_id, parsed.user_id);
        write_task.abort();
    }

    fn register(&self, user_id: Uuid, role: String, sender: UnboundedSender<Vec<u8>>) -> u64 {
        let mut state = self.state.write();
        state.next_id += 1;
        let id = state.next_id;
        state.clients.insert(
            id,
            Client {
                user_id,
                role,
                sender,
            },
        );
        state.user_index.entry(user_id).or_default().insert(id);
        let count = state.user_index.len() as i64;
        drop(state);
        self.metrics.set_ws_unique_user_count(count);
        self.broadcast_online_users();
        self.broadcast_online_count_to_admins();
        id
    }

    fn unregister(&self, id: u64, user_id: Uuid) {
        let mut state = self.state.write();
        state.clients.remove(&id);
        if let Some(set) = state.user_index.get_mut(&user_id) {
            set.remove(&id);
            if set.is_empty() {
                state.user_index.remove(&user_id);
            }
        }
        let count = state.user_index.len() as i64;
        drop(state);
        self.metrics.set_ws_unique_user_count(count);
        self.broadcast_online_users();
        self.broadcast_online_count_to_admins();
    }

    pub fn broadcast<T: Serialize>(&self, kind: &str, data: &T) {
        let envelope = Envelope { kind, data };
        let payload = match serde_json::to_vec(&envelope) {
            Ok(v) => v,
            Err(_) => return,
        };
        let state = self.state.read();
        for client in state.clients.values() {
            let _ = client.sender.send(payload.clone());
        }
    }

    pub fn send_to_user<T: Serialize>(&self, user_id: Uuid, kind: &str, data: &T) {
        let envelope = Envelope { kind, data };
        let payload = match serde_json::to_vec(&envelope) {
            Ok(v) => v,
            Err(_) => return,
        };
        let state = self.state.read();
        if let Some(ids) = state.user_index.get(&user_id) {
            for id in ids {
                if let Some(client) = state.clients.get(id) {
                    let _ = client.sender.send(payload.clone());
                }
            }
        }
    }

    pub fn unique_user_count(&self) -> usize {
        self.state.read().user_index.len()
    }

    pub fn online_user_ids(&self) -> Vec<Uuid> {
        self.state.read().user_index.keys().copied().collect()
    }

    fn online_users_payload(&self) -> Vec<u8> {
        let ids: Vec<String> = self
            .state
            .read()
            .user_index
            .keys()
            .map(|u| u.to_string())
            .collect();
        let envelope = Envelope {
            kind: "online_users",
            data: &serde_json::json!({"user_ids": ids}),
        };
        serde_json::to_vec(&envelope).unwrap_or_default()
    }

    fn online_count_payload(&self) -> Vec<u8> {
        let count = self.state.read().user_index.len();
        let envelope = Envelope {
            kind: "online_count",
            data: &serde_json::json!({"count": count}),
        };
        serde_json::to_vec(&envelope).unwrap_or_default()
    }

    fn broadcast_online_users(&self) {
        let payload = self.online_users_payload();
        let state = self.state.read();
        for client in state.clients.values() {
            let _ = client.sender.send(payload.clone());
        }
    }

    fn broadcast_online_count_to_admins(&self) {
        let payload = self.online_count_payload();
        let state = self.state.read();
        for client in state.clients.values() {
            if client.role == "admin" {
                let _ = client.sender.send(payload.clone());
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct AuthFrame {
    #[serde(rename = "type")]
    kind: String,
    token: String,
}

use futures::stream::StreamExt;
use futures::SinkExt;
