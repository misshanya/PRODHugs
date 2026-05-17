mod bot;
mod client;
mod linkstore;
mod loginstore;
mod notifier;

pub use bot::Bot;
pub use client::{Client, InlineKeyboardButton, InlineKeyboardMarkup, Update};
pub use linkstore::LinkStore;
pub use loginstore::{LoginSessionStatus, LoginStore, PollResult, TelegramUserInfo};
pub use notifier::Notifier;

pub fn deep_link_url(bot_username: &str, token: &str) -> String {
    format!("https://t.me/{bot_username}?start={token}")
}
