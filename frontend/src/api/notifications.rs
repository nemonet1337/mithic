use super::client::{ApiError, request};
use shared::Notification;

pub async fn fetch_notifications(
    token: &str,
    since_id: Option<&str>,
) -> Result<Vec<Notification>, ApiError> {
    let path = match since_id {
        Some(id) => format!("notifications?limit=20&cursor={}", id),
        None => "notifications?limit=20".into(),
    };
    request::<Vec<Notification>, ()>("GET", &path, Some(token), None).await
}

pub async fn mark_all_read(token: &str) -> Result<(), ApiError> {
    request::<(), ()>("POST", "notifications/read-all", Some(token), None).await
}
