//! Web Push API endpoints
//!
//! Provides API for managing Web Push subscriptions and VAPID keys.

use axum::{
    extract::State,
    Json,
};
use tracing::{error, info};

use crate::{
    error::{AppError, Result},
    models::{CreatePushSubscriptionRequest, PushSubscription, PushSubscriptionResponse},
    state::{AppState, AuthUser},
};

/// Get VAPID public key
///
/// Returns the VAPID public key that clients should use to subscribe to push notifications.
/// Returns 404 if push notifications are not configured.
pub async fn get_vapid_public_key(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let web_push_service = state.web_push_service();
    
    match web_push_service.get_public_key() {
        Some(key) => Ok(Json(serde_json::json!({
            "public_key": key
        }))),
        None => Err(AppError::NotFound("Push notifications not configured".to_string())),
    }
}

/// Register push subscription
///
/// Registers a new Web Push subscription for the authenticated user.
/// If a subscription with the same endpoint already exists, it will be updated.
pub async fn register_subscription(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreatePushSubscriptionRequest>,
) -> Result<Json<PushSubscriptionResponse>> {
    let user_id = auth_user.user_id;
    
    // Check if subscription already exists for this endpoint
    let existing: Option<PushSubscription> = state.surreal()
        .query("SELECT * FROM push_subscription WHERE user_id = $user_id AND endpoint = $endpoint")
        .bind(("user_id", user_id.to_string()))
        .bind(("endpoint", request.endpoint.clone()))
        .await
        .and_then(|mut res| res.take(0))
        .ok()
        .flatten();
    
    let subscription = if let Some(mut existing) = existing {
        // Update existing subscription with new keys
        info!("Updating existing push subscription for user {}", user_id);
        
        state.surreal()
            .query(r#"
                UPDATE push_subscription SET 
                    p256dh = $p256dh,
                    auth = $auth,
                    updated_at = time::now()
                WHERE id = $id
            "#)
            .bind(("id", existing.id.to_string()))
            .bind(("p256dh", request.p256dh.clone()))
            .bind(("auth", request.auth.clone()))
            .await
            .map_err(|e| AppError::Database(e))?;
        
        existing.p256dh = request.p256dh;
        existing.auth = request.auth;
        existing
    } else {
        // Create new subscription
        let subscription = PushSubscription::new(
            user_id,
            request.endpoint,
            request.p256dh,
            request.auth,
        );
        
        info!("Creating new push subscription for user {}", user_id);
        
        state.surreal()
            .create::<Option<PushSubscription>>(("push_subscription", subscription.id.to_string()))
            .content(subscription.clone())
            .await
            .map_err(|e| AppError::Database(e))?;
        
        subscription
    };
    
    Ok(Json(subscription.into()))
}

/// Unregister push subscription
///
/// Removes a Web Push subscription for the authenticated user.
/// Can delete by endpoint or all subscriptions.
pub async fn unregister_subscription(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let user_id = auth_user.user_id;
    
    // Check if endpoint is specified
    if let Some(endpoint) = request.get("endpoint").and_then(|v| v.as_str()) {
        // Delete specific subscription
        state.surreal()
            .query("DELETE push_subscription WHERE user_id = $user_id AND endpoint = $endpoint")
            .bind(("user_id", user_id.to_string()))
            .bind(("endpoint", endpoint.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;
        
        info!("Deleted push subscription for user {} with endpoint {}", user_id, endpoint);
    } else {
        // Delete all subscriptions for user
        state.surreal()
            .query("DELETE push_subscription WHERE user_id = $user_id")
            .bind(("user_id", user_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;
        
        info!("Deleted all push subscriptions for user {}", user_id);
    }
    
    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// Get user subscriptions
///
/// Returns all push subscriptions for the authenticated user.
pub async fn get_subscriptions(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<PushSubscriptionResponse>>> {
    let user_id = auth_user.user_id;
    
    let subscriptions: Vec<PushSubscription> = state.surreal()
        .query("SELECT * FROM push_subscription WHERE user_id = $user_id")
        .bind(("user_id", user_id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();
    
    let responses: Vec<PushSubscriptionResponse> = subscriptions
        .into_iter()
        .map(|s| s.into())
        .collect();
    
    Ok(Json(responses))
}

/// Send test push notification
///
/// Sends a test push notification to the authenticated user's devices.
/// Useful for verifying push notification setup.
pub async fn send_test_notification(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let user_id = auth_user.user_id;
    
    // Get user info for the notification
    let actor: Option<crate::models::Actor> = state.surreal()
        .query("SELECT * FROM actor WHERE id = $id")
        .bind(("id", user_id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .ok()
        .flatten();
    
    let actor_name = actor
        .as_ref()
        .map(|a| a.username.clone())
        .unwrap_or_else(|| "Someone".to_string());
    
    // Send test notification
    match state.web_push_service()
        .send_mention(
            state.surreal(),
            &user_id,
            &actor_name,
            &format!("{}/notifications", state.config().instance_url),
        )
        .await
    {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "Test notification sent"
        }))),
        Err(e) => {
            error!("Failed to send test notification: {}", e);
            Err(e)
        }
    }
}
