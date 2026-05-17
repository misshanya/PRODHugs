use std::sync::Arc;

use crate::telegram::Notifier;
use crate::ws::Hub;

pub mod hug;
pub mod note;
pub mod user;

pub use hug::HugService;
pub use note::NoteService;
pub use user::UserService;

/// Wires hub broadcasts and Telegram notifier into the services. Mirrors the
/// `SetHug*Callback` / `SetAnnouncement*Callback` set-up in the Go `app.New`.
pub fn wire_callbacks(
    hug: Arc<HugService>,
    user: Arc<UserService>,
    hub: Arc<Hub>,
    notifier: Arc<Notifier>,
) {
    {
        let hub = hub.clone();
        let notifier = notifier.clone();
        hug.set_on_completed(Arc::new(move |item, bonus, comment| {
            hub.broadcast("hug_completed", &crate::http::dto::ws::feed_dto(item));
            let n = notifier.clone();
            let giver = item.giver_id;
            let receiver = item.receiver_id;
            let hug_type = item.hug_type.clone();
            let comment = comment.clone();
            tokio::spawn(async move {
                n.notify_hug_completed(giver, receiver, &hug_type, bonus, comment.as_deref())
                    .await;
            });
        }));
    }
    {
        let hub = hub.clone();
        let notifier = notifier.clone();
        hug.set_on_suggestion(Arc::new(move |target, item, comment| {
            hub.send_to_user(
                target,
                "hug_suggestion",
                &crate::http::dto::ws::pending_dto(item),
            );
            let n = notifier.clone();
            let id = item.id;
            let giver = item.giver_id;
            let hug_type = item.hug_type.clone();
            let comment = comment.clone();
            tokio::spawn(async move {
                n.notify_hug_suggestion(target, id, giver, &hug_type, comment.as_deref())
                    .await;
            });
        }));
    }
    {
        let hub = hub.clone();
        let notifier = notifier.clone();
        hug.set_on_declined(Arc::new(move |target, hug_id, receiver_id| {
            hub.send_to_user(
                target,
                "hug_declined",
                &serde_json::json!({
                    "hug_id": hug_id.to_string(),
                    "receiver_id": receiver_id.to_string(),
                }),
            );
            let n = notifier.clone();
            tokio::spawn(async move {
                n.notify_hug_declined(target, receiver_id).await;
            });
        }));
    }
    {
        let hub = hub.clone();
        hug.set_on_cancelled(Arc::new(move |target, hug_id| {
            hub.send_to_user(
                target,
                "hug_cancelled",
                &serde_json::json!({"hug_id": hug_id.to_string()}),
            );
        }));
    }
    {
        let hub = hub.clone();
        user.set_on_announcement_created(Arc::new(move |ann| {
            hub.broadcast(
                "announcement",
                &serde_json::json!({
                    "id": ann.id.to_string(),
                    "message": ann.message,
                    "created_at": ann.created_at.to_rfc3339(),
                }),
            );
        }));
    }
    {
        let hub = hub.clone();
        user.set_on_announcement_removed(Arc::new(move |id| {
            hub.broadcast(
                "announcement_removed",
                &serde_json::json!({"id": id.to_string()}),
            );
        }));
    }
    {
        let hub = hub.clone();
        user.set_on_promotion_updated(Arc::new(move || {
            hub.broadcast::<()>("vips_updated", &());
        }));
    }
}
