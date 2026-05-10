//! Chart/Statistics API endpoints
//!
//! Provides API for viewing instance and user statistics.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use tracing::info;

use crate::{
    error::{AppError, Result},
    models::{ChartDataResponse, ChartQuery, ChartSpan, ChartType, InstanceStatsResponse, UserStatsResponse},
    state::{AppState, AuthUser},
};

/// Get instance statistics
///
/// Returns current snapshot of instance-wide statistics.
pub async fn get_instance_stats(
    State(state): State<AppState>,
) -> Result<Json<InstanceStatsResponse>> {
    let stats = state
        .chart_service()
        .get_instance_stats()
        .await?;

    Ok(Json(stats))
}

/// Get instance users chart
///
/// Returns time-series data for user count.
pub async fn get_users_chart(
    Query(query): Query<ChartQuery>,
    State(state): State<AppState>,
) -> Result<Json<ChartDataResponse>> {
    let span: ChartSpan = query.span.into();
    let data = state
        .chart_service()
        .get_chart_data(ChartType::Users, span, query.limit)
        .await?;

    Ok(Json(data))
}

/// Get instance notes chart
///
/// Returns time-series data for note count.
pub async fn get_notes_chart(
    Query(query): Query<ChartQuery>,
    State(state): State<AppState>,
) -> Result<Json<ChartDataResponse>> {
    let span: ChartSpan = query.span.into();
    let data = state
        .chart_service()
        .get_chart_data(ChartType::Notes, span, query.limit)
        .await?;

    Ok(Json(data))
}

/// Get instance drive chart
///
/// Returns time-series data for drive usage.
pub async fn get_drive_chart(
    Query(query): Query<ChartQuery>,
    State(state): State<AppState>,
) -> Result<Json<ChartDataResponse>> {
    let span: ChartSpan = query.span.into();
    let data = state
        .chart_service()
        .get_chart_data(ChartType::Drive, span, query.limit)
        .await?;

    Ok(Json(data))
}

/// Get federation chart
///
/// Returns time-series data for federation stats.
pub async fn get_federation_chart(
    Query(query): Query<ChartQuery>,
    State(state): State<AppState>,
) -> Result<Json<ChartDataResponse>> {
    let span: ChartSpan = query.span.into();
    let data = state
        .chart_service()
        .get_chart_data(ChartType::Federation, span, query.limit)
        .await?;

    Ok(Json(data))
}

/// Get user statistics
///
/// Returns current statistics for the authenticated user.
pub async fn get_my_stats(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<UserStatsResponse>> {
    let stats = state
        .chart_service()
        .get_user_stats(&auth_user.user_id.into())
        .await?;

    Ok(Json(stats))
}

/// Get user notes chart
///
/// Returns time-series data for the user's note count.
pub async fn get_my_notes_chart(
    auth_user: AuthUser,
    Query(query): Query<ChartQuery>,
    State(state): State<AppState>,
) -> Result<Json<ChartDataResponse>> {
    let span: ChartSpan = query.span.into();
    let data = state
        .chart_service()
        .get_user_chart_data(ChartType::PerUserNotes, &auth_user.user_id.into(), span, query.limit)
        .await?;

    Ok(Json(data))
}

/// Get user following chart
///
/// Returns time-series data for the user's following count.
pub async fn get_my_following_chart(
    auth_user: AuthUser,
    Query(query): Query<ChartQuery>,
    State(state): State<AppState>,
) -> Result<Json<ChartDataResponse>> {
    let span: ChartSpan = query.span.into();
    let data = state
        .chart_service()
        .get_user_chart_data(ChartType::PerUserFollowing, &auth_user.user_id.into(), span, query.limit)
        .await?;

    Ok(Json(data))
}

/// Get user drive chart
///
/// Returns time-series data for the user's drive usage.
pub async fn get_my_drive_chart(
    auth_user: AuthUser,
    Query(query): Query<ChartQuery>,
    State(state): State<AppState>,
) -> Result<Json<ChartDataResponse>> {
    let span: ChartSpan = query.span.into();
    let data = state
        .chart_service()
        .get_user_chart_data(ChartType::PerUserDrive, &auth_user.user_id.into(), span, query.limit)
        .await?;

    Ok(Json(data))
}
