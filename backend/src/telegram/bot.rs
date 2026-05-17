//! Long-polling Telegram bot — handles `/start <token>` deep-links for both
//! account-linking and Telegram-login flows, plus inline-button callbacks for
//! hug accept/decline from notification messages.

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::error::AppError;
use crate::repo;
use crate::service::{HugService, UserService};
use crate::telegram::client::{
    CallbackQuery, Client, InlineKeyboardButton, InlineKeyboardMarkup, Message, Update,
};
use crate::telegram::{LinkStore, LoginStore, TelegramUserInfo};

pub struct Bot {
    client: Arc<Client>,
    link_store: Arc<LinkStore>,
    login_store: Arc<LoginStore>,
    pool: PgPool,
    hug: Arc<HugService>,
    user: Arc<UserService>,
}

impl Bot {
    pub fn new(
        client: Arc<Client>,
        link_store: Arc<LinkStore>,
        login_store: Arc<LoginStore>,
        pool: PgPool,
        hug: Arc<HugService>,
        user: Arc<UserService>,
    ) -> Self {
        Self {
            client,
            link_store,
            login_store,
            pool,
            hug,
            user,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Run long-polling until the task is aborted.
    pub async fn run(self: Arc<Self>) {
        if !self.client.enabled() {
            info!("telegram bot disabled (no token)");
            return;
        }
        info!("telegram bot started (long-polling)");
        let mut offset: i64 = 0;
        loop {
            match self.client.get_updates(offset, 30).await {
                Ok(updates) => {
                    for update in updates {
                        offset = update.update_id + 1;
                        let me = self.clone();
                        tokio::spawn(async move {
                            me.dispatch(update).await;
                        });
                    }
                }
                Err(err) => {
                    warn!(%err, "telegram bot: getUpdates failed");
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
        }
    }

    pub async fn send_hug_suggestion(
        &self,
        receiver_id: Uuid,
        hug_id: Uuid,
        giver_name: &str,
        phrase: &str,
        comment: Option<&str>,
    ) {
        let telegram_id = match repo::user::get_telegram_id(&self.pool, receiver_id).await {
            Ok(Some(id)) => id,
            _ => return,
        };
        let mut text = format!("🤗 <b>{}</b> {}!", escape_html(giver_name), phrase);
        if let Some(c) = comment {
            if !c.is_empty() {
                text.push_str(&format!("\n\n💬 <i>{}</i>", escape_html(c)));
            }
        }
        let keyboard = InlineKeyboardMarkup {
            inline_keyboard: vec![vec![
                InlineKeyboardButton {
                    text: "Обнять 🤗".into(),
                    callback_data: Some(format!("accept:{hug_id}")),
                },
                InlineKeyboardButton {
                    text: "Отклонить".into(),
                    callback_data: Some(format!("decline:{hug_id}")),
                },
            ]],
        };
        if let Err(err) = self
            .client
            .send_message_with_kb(telegram_id, &text, &keyboard)
            .await
        {
            error!(%err, "telegram bot: send_hug_suggestion failed");
        }
    }

    async fn dispatch(self: Arc<Self>, update: Update) {
        if let Some(cb) = update.callback_query {
            self.handle_callback(cb).await;
            return;
        }
        if let Some(msg) = update.message {
            if msg.text.as_deref().map(|t| t.starts_with("/start")).unwrap_or(false) {
                self.handle_start(msg).await;
            }
        }
    }

    async fn handle_start(&self, msg: Message) {
        let chat_id = msg.chat.id;
        let text = msg.text.clone().unwrap_or_default();
        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        if parts.len() < 2 || parts[1].trim().is_empty() {
            self.reply(chat_id, "Чтобы привязать аккаунт, используй настройки на сайте").await;
            return;
        }
        let token = parts[1].trim();

        if let Some(rest) = token.strip_prefix("login_") {
            self.handle_login_start(&msg, rest).await;
            return;
        }

        let user_id = match self.link_store.consume_token(token) {
            Some(id) => id,
            None => {
                self.reply(chat_id, "Ссылка недействительна или истекла. Попробуй снова через настройки приложения").await;
                return;
            }
        };

        let taken = match repo::user::is_telegram_id_taken(&self.pool, chat_id, user_id).await {
            Ok(t) => t,
            Err(err) => {
                error!(?err, "telegram bot: failed to check telegram_id");
                self.reply(chat_id, "Произошла ошибка. Попробуй позже :(").await;
                return;
            }
        };
        if taken {
            self.reply(chat_id, "Этот Telegram аккаунт уже привязан к другому пользователю :(").await;
            return;
        }

        if let Err(err) = repo::user::set_telegram_id(&self.pool, user_id, chat_id).await {
            error!(?err, %user_id, %chat_id, "telegram bot: failed to set telegram_id");
            self.reply(chat_id, "Произошла ошибка при привязке. Попробуй позже :(").await;
            return;
        }
        info!(%user_id, %chat_id, "telegram bot: account linked");
        self.reply(chat_id, "✅ Аккаунт привязан! Теперь ты не пропустишь обнимашки от любимых продовцев").await;
    }

    async fn handle_login_start(&self, msg: &Message, bot_token: &str) {
        let chat_id = msg.chat.id;
        let from = match &msg.from {
            Some(u) => u.clone(),
            None => {
                self.reply(chat_id, "Не удалось определить отправителя").await;
                return;
            }
        };

        let poll_token = match self.login_store.consume_bot_token(bot_token) {
            Some(p) => p,
            None => {
                self.reply(chat_id, "Ссылка недействительна или истекла. Попробуй снова").await;
                return;
            }
        };

        let info = TelegramUserInfo {
            telegram_id: chat_id,
            username: from.username.unwrap_or_default(),
            first_name: from.first_name,
            last_name: from.last_name.unwrap_or_default(),
        };

        match self.user.login_via_telegram(&info).await {
            Ok(user) => {
                self.login_store.authenticate(&poll_token, user.id);
                info!(%user.id, %chat_id, "telegram bot: login successful");
                self.reply(chat_id, "✅ Вход выполнен! Можешь вернуться в приложение").await;
            }
            Err(err) => {
                let friendly = friendly_login_error(&err);
                self.login_store
                    .fail(&poll_token, friendly.clone());
                error!(?err, %chat_id, "telegram bot: login failed");
                self.reply(chat_id, &format!("Не удалось войти: {friendly}")).await;
            }
        }
    }

    async fn handle_callback(&self, cb: CallbackQuery) {
        let data = match &cb.data {
            Some(d) => d.clone(),
            None => return,
        };
        let chat_id = cb
            .message
            .as_ref()
            .map(|m| m.chat.id)
            .unwrap_or(cb.from.id);
        let (action, id_str) = if let Some(rest) = data.strip_prefix("accept:") {
            ("accept", rest)
        } else if let Some(rest) = data.strip_prefix("decline:") {
            ("decline", rest)
        } else {
            return;
        };
        let Ok(hug_id) = Uuid::parse_str(id_str) else {
            self.answer_callback(&cb.id, "Ошибка: некорректные данные").await;
            return;
        };

        let user = match repo::user::get_by_telegram_id(&self.pool, chat_id).await {
            Ok(u) => u,
            Err(_) => {
                self.answer_callback(&cb.id, "Ваш Telegram не привязан к аккаунту").await;
                return;
            }
        };

        let original_text = cb.message.as_ref().and_then(|m| m.text.clone()).unwrap_or_default();
        let msg_id = cb.message.as_ref().map(|m| m.message_id).unwrap_or(0);

        match action {
            "accept" => match self.hug.accept_hug(hug_id, user.id).await {
                Ok(_) => {
                    self.answer_callback(&cb.id, "Объятие принято! 🤗").await;
                    let new_text = format!("{}\n\n✅ <b>Принято!</b>", escape_html(&original_text));
                    let _ = self
                        .client
                        .edit_message_text_html(chat_id, msg_id, &new_text)
                        .await;
                }
                Err(err) => {
                    error!(?err, %hug_id, "telegram bot: accept failed");
                    self.answer_callback(&cb.id, &format!("Не удалось принять: {}", friendly_hug_error(&err))).await;
                }
            },
            "decline" => match self.hug.decline_hug(hug_id, user.id).await {
                Ok(_) => {
                    self.answer_callback(&cb.id, "Объятие отклонено").await;
                    let new_text =
                        format!("{}\n\n❌ <b>Отклонено</b>", escape_html(&original_text));
                    let _ = self
                        .client
                        .edit_message_text_html(chat_id, msg_id, &new_text)
                        .await;
                }
                Err(err) => {
                    error!(?err, %hug_id, "telegram bot: decline failed");
                    self.answer_callback(&cb.id, &format!("Не удалось отклонить: {}", friendly_hug_error(&err))).await;
                }
            },
            _ => {}
        }
    }

    async fn reply(&self, chat_id: i64, text: &str) {
        if let Err(err) = self.client.send_message_html(chat_id, text).await {
            error!(%err, %chat_id, "telegram bot: failed to send message");
        }
    }

    async fn answer_callback(&self, callback_id: &str, text: &str) {
        if let Err(err) = self.client.answer_callback(callback_id, text).await {
            error!(%err, "telegram bot: failed to answer callback");
        }
    }
}

fn friendly_hug_error(err: &AppError) -> String {
    match err {
        AppError::HugNotFound => "объятие не найдено".into(),
        AppError::HugNotPending => "объятие уже обработано".into(),
        AppError::HugExpired => "объятие истекло".into(),
        _ => "попробуйте позже".into(),
    }
}

fn friendly_login_error(err: &AppError) -> String {
    match err {
        AppError::UserBanned => "ваш аккаунт заблокирован".into(),
        _ => "попробуйте позже".into(),
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
