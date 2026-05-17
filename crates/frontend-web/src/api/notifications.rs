use crate::models::Notification;
use super::client::{ApiError, request};

pub async fn fetch_notifications(
    token: &str,
    since_id: Option<&str>,
) -> Result<Vec<Notification>, ApiError> {
    let path = match since_id {
        Some(id) => format!("v1/notifications?limit=20&since_id={}", id),
        None     => "v1/notifications?limit=20".into(),
    };
    request::<Vec<Notification>, ()>("GET", &path, Some(token), None).await
}

pub async fn mark_all_read(token: &str) -> Result<(), ApiError> {
    request::<(), ()>("POST", "v1/notifications/read-all", Some(token), None).await
}

pub async fn mark_read(token: &str, id: &str) -> Result<(), ApiError> {
    request::<(), ()>("POST", &format!("v1/notifications/{}/read", id), Some(token), None).await
}
