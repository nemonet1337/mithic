//! Web Push delivery (VAPID + encrypted payload)

use mithic_core::models::actor::ActorId;
use mithic_db::queries::{
    PushSubscription, delete_push_subscription_by_endpoint, list_push_subscriptions,
};
use shared::Notification as NotifDto;
use tracing::{debug, warn};
use web_push::{
    ContentEncoding, SubscriptionInfo, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

use crate::state::AppState;

/// JSON payload shown by the service worker `push` handler
fn notification_payload(dto: &NotifDto) -> String {
    let title = match dto.notification_type {
        shared::NotificationType::Mention => "Mention",
        shared::NotificationType::Reply => "Reply",
        shared::NotificationType::Renote => "Renote",
        shared::NotificationType::Quote => "Quote",
        shared::NotificationType::Reaction => "Reaction",
        shared::NotificationType::Follow => "New follower",
        shared::NotificationType::FollowRequest => "Follow request",
        shared::NotificationType::FollowRequestAccepted => "Follow accepted",
        shared::NotificationType::PollEnded => "Poll ended",
        shared::NotificationType::UserSignup => "Signup",
    };
    let body = dto
        .sender
        .as_ref()
        .map(|u| {
            let name = u.display_name.as_deref().unwrap_or(&u.username);
            format!("@{name}")
        })
        .unwrap_or_else(|| title.to_string());

    serde_json::json!({
        "title": title,
        "body": body,
        "tag": dto.id,
        "url": "/notifications",
        "notificationId": dto.id,
    })
    .to_string()
}

async fn send_one(
    state: &AppState,
    sub: &PushSubscription,
    payload: &str,
) -> Result<(), web_push::WebPushError> {
    let Some(private_key) = state.config().vapid_private_key.as_deref() else {
        return Ok(());
    };
    let Some(client) = state.web_push_client() else {
        return Ok(());
    };

    let info = SubscriptionInfo::new(&sub.endpoint, &sub.p256dh, &sub.auth);
    let mut sig_builder = VapidSignatureBuilder::from_base64(private_key, &info)?;
    sig_builder.add_claim("sub", state.config().vapid_contact.as_str());
    let signature = sig_builder.build()?;

    let mut builder = WebPushMessageBuilder::new(&info);
    builder.set_payload(ContentEncoding::Aes128Gcm, payload.as_bytes());
    builder.set_vapid_signature(signature);
    let message = builder.build()?;

    client.send(message).await
}

/// Fan-out Web Push for a recipient. Runs best-effort; never blocks the caller path long
/// when spawned.
pub async fn deliver_web_push(state: &AppState, recipient_id: ActorId, dto: &NotifDto) {
    if state.config().vapid_private_key.is_none() {
        return;
    }

    let subs = match list_push_subscriptions(state.surreal(), &recipient_id).await {
        Ok(s) if !s.is_empty() => s,
        Ok(_) => return,
        Err(e) => {
            warn!("list_push_subscriptions failed: {e}");
            return;
        }
    };

    let payload = notification_payload(dto);
    for sub in subs {
        match send_one(state, &sub, &payload).await {
            Ok(()) => debug!("Web push sent to {}", sub.endpoint),
            Err(e) => match e {
                web_push::WebPushError::EndpointNotValid(_)
                | web_push::WebPushError::EndpointNotFound(_) => {
                    let _ =
                        delete_push_subscription_by_endpoint(state.surreal(), &sub.endpoint).await;
                    debug!("Removed stale push subscription {}", sub.endpoint);
                }
                other => warn!("Web push failed for {}: {other}", sub.endpoint),
            },
        }
    }
}
