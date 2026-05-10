//! Instance configuration API endpoints
//!
//! Admin-only endpoints for managing instance settings.

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use tracing::info;

use crate::{
    error::{AppError, Result},
    models::{FederatedInstance, FederatedInstanceResponse, InstanceConfigResponse, UpdateInstanceConfigRequest},
    services::instance::InstanceStats,
    state::{AppState, AuthUser},
};

#[derive(Debug, Deserialize)]
pub struct FederatedInstancesQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub host: Option<String>,
    pub blocked: Option<bool>,
    pub suspended: Option<bool>,
    pub not_responding: Option<bool>,
    pub sort: Option<String>,
}

/// Get instance configuration (admin only)
pub async fn get_instance_config(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<InstanceConfigResponse>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let config = state.instance_service().get_config().await?;

    Ok(Json(InstanceConfigResponse::from(config)))
}

/// Update instance configuration (admin only)
pub async fn update_instance_config(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateInstanceConfigRequest>,
) -> Result<Json<InstanceConfigResponse>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let config = state.instance_service().update_config(req).await?;

    info!("Instance configuration updated by admin {}", auth_user.user_id);

    Ok(Json(InstanceConfigResponse::from(config)))
}

/// Get instance statistics (public)
pub async fn get_instance_stats(
    State(state): State<AppState>,
) -> Result<Json<InstanceStats>> {
    let stats = state.instance_service().get_stats().await?;

    Ok(Json(stats))
}

/// Get federated instances (admin only)
pub async fn get_federated_instances(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<FederatedInstancesQuery>,
) -> Result<Json<Vec<FederatedInstanceResponse>>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut query_str = "SELECT * FROM federated_instance LIMIT $limit START $offset".to_string();
    let mut bindings: Vec<(&str, String)> = vec![("limit", limit.to_string()), ("offset", offset.to_string())];

    // Add filters
    if let Some(host) = &query.host {
        query_str.push_str(" AND host ~ $host");
        bindings.push(("host", host.clone()));
    }

    if let Some(blocked) = query.blocked {
        query_str.push_str(&format!(" AND is_blocked = {}", blocked));
    }

    if let Some(suspended) = query.suspended {
        query_str.push_str(&format!(" AND is_suspended = {}", suspended));
    }

    if let Some(not_responding) = query.not_responding {
        query_str.push_str(&format!(" AND is_not_responding = {}", not_responding));
    }

    // Add sorting
    let sort_order = match query.sort.as_deref() {
        Some("last_retrieved_at") => "ORDER BY last_retrieved_at DESC",
        Some("first_retrieved_at") => "ORDER BY first_retrieved_at DESC",
        Some("host") => "ORDER BY host ASC",
        _ => "ORDER BY last_retrieved_at DESC",
    };
    query_str.push_str(sort_order);

    let mut surreal_query = state.surreal().query(&query_str);
    for (key, value) in bindings {
        surreal_query = surreal_query.bind((key, value));
    }

    let instances: Vec<FederatedInstance> = surreal_query
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    let responses: Vec<FederatedInstanceResponse> = instances
        .into_iter()
        .map(FederatedInstanceResponse::from)
        .collect();

    Ok(Json(responses))
}

/// 連合先からのフォロワー
pub async fn get_federation_followers(
    State(state): State<AppState>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let host = query.get("host")
        .ok_or_else(|| AppError::Validation("Host parameter required".to_string()))?;

    let limit: usize = query.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50);

    let query_str = r#"
        SELECT * FROM follow WHERE out IN (SELECT VALUE id FROM user WHERE host = $host) LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(query_str)
        .bind(("host", host.clone()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Database(e))?;

    let follows: Vec<serde_json::Value> = result.take(0).unwrap_or_default();

    Ok(Json(follows))
}

/// 連合先へのフォロー
pub async fn get_federation_following(
    State(state): State<AppState>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let host = query.get("host")
        .ok_or_else(|| AppError::Validation("Host parameter required".to_string()))?;

    let limit: usize = query.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50);

    let query_str = r#"
        SELECT * FROM follow WHERE in IN (SELECT VALUE id FROM user WHERE host = $host) LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(query_str)
        .bind(("host", host.clone()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Database(e))?;

    let follows: Vec<serde_json::Value> = result.take(0).unwrap_or_default();

    Ok(Json(follows))
}

/// 連合ユーザー一覧
pub async fn get_federation_users(
    State(state): State<AppState>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let host = query.get("host")
        .ok_or_else(|| AppError::Validation("Host parameter required".to_string()))?;

    let limit: usize = query.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50);

    let query_str = r#"
        SELECT * FROM user WHERE host = $host LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(query_str)
        .bind(("host", host.clone()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Database(e))?;

    let users: Vec<serde_json::Value> = result.take(0).unwrap_or_default();

    Ok(Json(users))
}

/// インスタンス詳細
pub async fn show_instance(
    State(state): State<AppState>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>> {
    let host = query.get("host")
        .ok_or_else(|| AppError::Validation("Host parameter required".to_string()))?;

    let query_str = r#"
        SELECT * FROM federated_instance WHERE host = $host
    "#;

    let mut result = state
        .surreal()
        .query(query_str)
        .bind(("host", host.clone()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let instance: Option<serde_json::Value> = result.take(0).ok().flatten();
    let instance = instance.ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;

    Ok(Json(instance))
}

/// インスタンスメタ情報
pub async fn get_meta(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let config = state.config();

    Ok(Json(serde_json::json!({
        "name": config.instance_name,
        "description": config.instance_description,
        "maintainer_name": config.maintainer_name,
        "maintainer_email": config.maintainer_email,
        "url": config.instance_url,
        "version": "1.0.0",
        "registrations": config.registration_mode,
        "max_note_text_length": 5000,
    })))
}

/// 全体統計
pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // ユーザー数
    let users_count: Option<i64> = state
        .surreal()
        .query("SELECT count() FROM user GROUP BY count()")
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .ok()
        .flatten();

    // 投稿数
    let notes_count: Option<i64> = state
        .surreal()
        .query("SELECT count() FROM note GROUP BY count()")
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .ok()
        .flatten();

    // インスタンス数
    let instances_count: Option<i64> = state
        .surreal()
        .query("SELECT count() FROM federated_instance GROUP BY count()")
        .await
        .map_err(|e| AppError::Database(e))?
        .take(0)
        .ok()
        .flatten();

    Ok(Json(serde_json::json!({
        "users_count": users_count.unwrap_or(0),
        "notes_count": notes_count.unwrap_or(0),
        "instances_count": instances_count.unwrap_or(0),
    })))
}

