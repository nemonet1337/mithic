//! Web Push delivery (VAPID + encrypted payload)

use crate::db::queries::{
    PushSubscription, delete_push_subscription_by_endpoint, list_push_subscriptions,
};
use crate::models::actor::ActorId;
use base64::Engine;
use shared::Notification as NotifDto;
use tracing::{debug, warn};
use web_push_native::{p256::PublicKey, Auth, WebPushBuilder};

use crate::state::AppState;

enum PushSendError {
    Stale,
    Other(anyhow::Error),
}

fn decode_url_b64(s: &str) -> Result<Vec<u8>, anyhow::Error> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(s))
        .map_err(|e| anyhow::anyhow!("invalid base64: {e}"))
}

fn is_stale_push_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::NOT_FOUND || status == reqwest::StatusCode::GONE
}

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
) -> Result<(), PushSendError> {
    let Some(key_pair) = state.vapid_key_pair() else {
        return Ok(());
    };

    let endpoint: http::Uri = sub
        .endpoint
        .parse()
        .map_err(|e| PushSendError::Other(anyhow::anyhow!("invalid endpoint: {e}")))?;
    let p256dh = decode_url_b64(&sub.p256dh).map_err(PushSendError::Other)?;
    let auth = decode_url_b64(&sub.auth).map_err(PushSendError::Other)?;
    if auth.len() != 16 {
        return Err(PushSendError::Other(anyhow::anyhow!(
            "invalid push auth length {}",
            auth.len()
        )));
    }

    let ua_public = PublicKey::from_sec1_bytes(&p256dh)
        .map_err(|e| PushSendError::Other(anyhow::anyhow!("invalid p256dh: {e}")))?;
    let ua_auth = Auth::clone_from_slice(&auth);

    let request = WebPushBuilder::new(endpoint, ua_public, ua_auth)
        .with_vapid(key_pair, state.config().vapid_contact.as_str())
        .build(payload.as_bytes())
        .map_err(|e| PushSendError::Other(anyhow::anyhow!("web push build: {e}")))?;

    let (parts, body) = request.into_parts();
    let resp = state
        .http_client()
        .request(parts.method, parts.uri.to_string())
        .headers(parts.headers)
        .body(body)
        .send()
        .await
        .map_err(|e| PushSendError::Other(anyhow::anyhow!("web push send: {e}")))?;

    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else if is_stale_push_status(status) {
        Err(PushSendError::Stale)
    } else {
        Err(PushSendError::Other(anyhow::anyhow!(
            "push service returned {status}"
        )))
    }
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
            Err(PushSendError::Stale) => {
                let _ = delete_push_subscription_by_endpoint(state.surreal(), &sub.endpoint).await;
                debug!("Removed stale push subscription {}", sub.endpoint);
            }
            Err(PushSendError::Other(e)) => warn!("Web push failed for {}: {e}", sub.endpoint),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_on_404_and_410() {
        assert!(is_stale_push_status(reqwest::StatusCode::NOT_FOUND));
        assert!(is_stale_push_status(reqwest::StatusCode::GONE));
        assert!(!is_stale_push_status(reqwest::StatusCode::OK));
        assert!(!is_stale_push_status(reqwest::StatusCode::BAD_REQUEST));
    }

    #[test]
    fn decode_url_safe_b64() {
        let raw = decode_url_b64("RS0WdYWWo1HajXg3NZR1olzCf31i-ZBGDkFyCs7j1jw").unwrap();
        assert_eq!(raw.len(), 32);
    }

    #[test]
    fn build_request_from_example_subscription() {
        use web_push_native::jwt_simple::algorithms::ES256KeyPair;

        let raw = decode_url_b64("RS0WdYWWo1HajXg3NZR1olzCf31i-ZBGDkFyCs7j1jw").unwrap();
        let key_pair = ES256KeyPair::from_bytes(&raw).unwrap();
        let p256dh = decode_url_b64(
            "BLn9b-VR0ca83knDNZ32dCHGyjJp-1riX9ZTN40MqV8K_LpQmLqxC_DoHvqvFXO_nGdAB4W9dogZb_sM-uV4JbY",
        )
        .unwrap();
        let auth = decode_url_b64("_ordMnz7uTCmrpBTeUV4Bw").unwrap();
        let ua_public = PublicKey::from_sec1_bytes(&p256dh).unwrap();
        let ua_auth = Auth::clone_from_slice(&auth);
        let request = WebPushBuilder::new("https://example.com/".parse().unwrap(), ua_public, ua_auth)
            .with_vapid(&key_pair, "mailto:admin@example.com")
            .build(b"hello".as_slice())
            .unwrap();
        assert_eq!(request.method(), http::Method::POST);
        assert_eq!(
            request.headers().get(http::header::CONTENT_ENCODING).unwrap(),
            "aes128gcm"
        );
        assert!(request.headers().get(http::header::AUTHORIZATION).is_some());
        assert!(!request.body().is_empty());
    }
}
