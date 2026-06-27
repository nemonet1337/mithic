use mithic_core::models::actor::ActorId;
use mithic_core::{AppError, Result};
use mithic_db::SurrealClient;

pub async fn suspend_user(surreal: &SurrealClient, user_id: &ActorId) -> Result<()> {
    surreal
        .query("UPDATE user SET is_suspended = true WHERE id = type::record('user', $id);")
        .bind(("id", user_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn unsuspend_user(surreal: &SurrealClient, user_id: &ActorId) -> Result<()> {
    surreal
        .query("UPDATE user SET is_suspended = false WHERE id = type::record('user', $id);")
        .bind(("id", user_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub async fn is_suspended(surreal: &SurrealClient, user_id: &ActorId) -> Result<bool> {
    let result: Option<bool> = surreal
        .query("SELECT is_suspended FROM user WHERE id = type::record('user', $id);")
        .bind(("id", user_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .take(0)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(result.unwrap_or(false))
}