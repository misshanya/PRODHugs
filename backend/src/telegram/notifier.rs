//! Telegram notification helpers. All methods are fire-and-forget — errors
//! are logged but never propagated to the caller.

use std::sync::Arc;

use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::repo;
use crate::telegram::bot::Bot;
use crate::telegram::client::Client;

pub struct Notifier {
    client: Arc<Client>,
    bot: Arc<Bot>,
    pool: PgPool,
}

impl Notifier {
    pub fn new(client: Arc<Client>, bot: Arc<Bot>, pool: PgPool) -> Self {
        Self { client, bot, pool }
    }

    pub fn enabled(&self) -> bool {
        self.client.enabled()
    }

    pub async fn notify_hug_suggestion(
        &self,
        receiver_id: Uuid,
        hug_id: Uuid,
        giver_id: Uuid,
        hug_type: &str,
        comment: Option<&str>,
    ) {
        let giver = match repo::user::get_by_id(&self.pool, giver_id).await {
            Ok(u) => u,
            Err(err) => {
                error!(?err, %giver_id, "telegram: lookup giver failed");
                return;
            }
        };
        let name = display_name(&giver);
        let phrase = hug_type_suggestion_phrase(hug_type);
        self.bot
            .send_hug_suggestion(receiver_id, hug_id, &name, phrase, comment)
            .await;
    }

    pub async fn notify_hug_completed(
        &self,
        giver_id: Uuid,
        receiver_id: Uuid,
        hug_type: &str,
        bonus_coins: i32,
        comment: Option<&str>,
    ) {
        let (Ok(giver), Ok(receiver)) = (
            repo::user::get_by_id(&self.pool, giver_id).await,
            repo::user::get_by_id(&self.pool, receiver_id).await,
        ) else {
            return;
        };
        let total = 1 + bonus_coins;
        let coin_text = if bonus_coins > 0 {
            format!("+{total} (бонус +{bonus_coins})")
        } else {
            format!("+{total}")
        };
        let hug_word = hug_type_completed_noun(hug_type);
        let giver_coin_text = if comment.is_some() {
            "0 (оплата комментария)".to_string()
        } else {
            coin_text.clone()
        };
        let receiver_verb = gender_verb(receiver.gender.as_deref(), "принял", "приняла", "принял(а)");
        let mut giver_msg = format!(
            "🎉 <b>{}</b> {} {}! {} {}",
            display_name(&receiver),
            receiver_verb,
            hug_word,
            giver_coin_text,
            plural_obnimani(total),
        );
        if comment.is_some() {
            giver_msg = format!(
                "🎉 <b>{}</b> {} {}! {}",
                display_name(&receiver),
                receiver_verb,
                hug_word,
                giver_coin_text
            );
        }
        let mut receiver_msg = format!(
            "🎉 Вы обнялись с <b>{}</b>! {} {}",
            display_name(&giver),
            coin_text,
            plural_obnimani(total)
        );
        if let Some(c) = comment {
            if !c.is_empty() {
                receiver_msg.push_str(&format!("\n\n💬 <i>{}</i>", escape_html(c)));
            }
        }
        self.send_to_user(giver_id, &giver_msg).await;
        self.send_to_user(receiver_id, &receiver_msg).await;
    }

    pub async fn notify_hug_declined(&self, giver_id: Uuid, receiver_id: Uuid) {
        let Ok(receiver) = repo::user::get_by_id(&self.pool, receiver_id).await else {
            return;
        };
        let verb = gender_verb(receiver.gender.as_deref(), "отклонил", "отклонила", "отклонил(а)");
        let text = format!("😔 <b>{}</b> {} объятие", display_name(&receiver), verb);
        self.send_to_user(giver_id, &text).await;
    }

    pub async fn notify_hug_cancelled(&self, receiver_id: Uuid, giver_id: Uuid) {
        let Ok(giver) = repo::user::get_by_id(&self.pool, giver_id).await else {
            return;
        };
        let verb = gender_verb(giver.gender.as_deref(), "отменил", "отменила", "отменил(а)");
        let text = format!("❌ <b>{}</b> {} запрос на объятие", display_name(&giver), verb);
        self.send_to_user(receiver_id, &text).await;
    }

    async fn send_to_user(&self, user_id: Uuid, text: &str) {
        if !self.enabled() {
            return;
        }
        let telegram_id = match repo::user::get_telegram_id(&self.pool, user_id).await {
            Ok(Some(id)) => id,
            _ => return,
        };
        if let Err(err) = self.client.send_message_html(telegram_id, text).await {
            error!(%err, %user_id, %telegram_id, "telegram: send_message failed");
        }
    }
}

fn display_name(u: &crate::models::User) -> String {
    if let Some(ref d) = u.display_name {
        if !d.is_empty() {
            return escape_html(d);
        }
    }
    escape_html(&u.username)
}

fn hug_type_suggestion_phrase(hug_type: &str) -> &'static str {
    match hug_type {
        "bear" => "хочет обнять тебя по-медвежьи",
        "group" => "хочет обнять тебя вместе со всеми",
        "warm" => "хочет тепло тебя обнять",
        "soul" => "хочет обнять тебя по-душевному",
        _ => "хочет тебя обнять",
    }
}

fn hug_type_completed_noun(hug_type: &str) -> &'static str {
    match hug_type {
        "bear" => "медвежьи обнимашки",
        "group" => "групповые обнимашки",
        "warm" => "тёплые обнимашки",
        "soul" => "душевные обнимашки",
        _ => "обнимашки",
    }
}

fn gender_verb(gender: Option<&str>, male: &'static str, female: &'static str, fallback: &'static str) -> &'static str {
    match gender {
        Some("male") => male,
        Some("female") => female,
        _ => fallback,
    }
}

fn plural_obnimani(n: i32) -> &'static str {
    let abs = n.unsigned_abs();
    let mod10 = abs % 10;
    let mod100 = abs % 100;
    if mod10 == 1 && mod100 != 11 {
        "обниманя"
    } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
        "обнимани"
    } else {
        "обнимань"
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
