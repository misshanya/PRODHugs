//! Minimal Telegram Bot API client — just what the bot/notifier need.
//!
//! No-ops when the token is empty (feature disabled).

use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::Client as Http;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Client {
    token: String,
    http: Http,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    ok: bool,
    description: Option<String>,
    result: Option<T>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InlineKeyboardButton {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_data: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InlineKeyboardMarkup {
    pub inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: i64,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Chat {
    pub id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    #[serde(default)]
    pub message_id: i64,
    pub chat: Chat,
    #[serde(default)]
    pub from: Option<User>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    pub from: User,
    #[serde(default)]
    pub message: Option<Message>,
    #[serde(default)]
    pub data: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Update {
    pub update_id: i64,
    #[serde(default)]
    pub message: Option<Message>,
    #[serde(default)]
    pub callback_query: Option<CallbackQuery>,
}

impl Client {
    pub fn new(token: String) -> Self {
        let http = Http::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self { token, http }
    }

    pub fn enabled(&self) -> bool {
        !self.token.is_empty()
    }

    fn url(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.token, method)
    }

    pub async fn send_message_html(&self, chat_id: i64, text: &str) -> Result<()> {
        if !self.enabled() {
            return Ok(());
        }
        let body = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "HTML",
        });
        let _: ApiResponse<serde_json::Value> = self.post("sendMessage", &body).await?;
        Ok(())
    }

    pub async fn send_message_with_kb(
        &self,
        chat_id: i64,
        text: &str,
        keyboard: &InlineKeyboardMarkup,
    ) -> Result<()> {
        if !self.enabled() {
            return Ok(());
        }
        let body = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "HTML",
            "reply_markup": keyboard,
        });
        let _: ApiResponse<serde_json::Value> = self.post("sendMessage", &body).await?;
        Ok(())
    }

    pub async fn answer_callback(&self, callback_id: &str, text: &str) -> Result<()> {
        if !self.enabled() {
            return Ok(());
        }
        let body = serde_json::json!({
            "callback_query_id": callback_id,
            "text": text,
            "show_alert": false,
        });
        let _: ApiResponse<serde_json::Value> = self.post("answerCallbackQuery", &body).await?;
        Ok(())
    }

    pub async fn edit_message_text_html(
        &self,
        chat_id: i64,
        message_id: i64,
        text: &str,
    ) -> Result<()> {
        if !self.enabled() {
            return Ok(());
        }
        let body = serde_json::json!({
            "chat_id": chat_id,
            "message_id": message_id,
            "text": text,
            "parse_mode": "HTML",
        });
        let _: ApiResponse<serde_json::Value> = self.post("editMessageText", &body).await?;
        Ok(())
    }

    /// Fetch updates via long-polling.
    pub async fn get_updates(&self, offset: i64, timeout: i64) -> Result<Vec<Update>> {
        if !self.enabled() {
            return Ok(Vec::new());
        }
        let body = serde_json::json!({
            "offset": offset,
            "timeout": timeout,
            "allowed_updates": ["message", "callback_query"],
        });
        let resp: ApiResponse<Vec<Update>> = self.post("getUpdates", &body).await?;
        Ok(resp.result.unwrap_or_default())
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        body: &serde_json::Value,
    ) -> Result<ApiResponse<T>> {
        let resp = self
            .http
            .post(self.url(method))
            .json(body)
            .send()
            .await
            .map_err(|e| anyhow!("telegram {method}: {e}"))?;
        let api: ApiResponse<T> = resp
            .json()
            .await
            .map_err(|e| anyhow!("telegram {method}: decode: {e}"))?;
        if !api.ok {
            return Err(anyhow!(
                "telegram {method}: {}",
                api.description.as_deref().unwrap_or("unknown error")
            ));
        }
        Ok(api)
    }
}
