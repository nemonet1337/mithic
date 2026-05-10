//! Chart models for statistics
//!
//! Time-series data for instance and user statistics.

use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

/// Chart entry ID
pub type ChartEntryId = Ulid;

/// Time span for chart aggregation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChartSpan {
    Hour,
    Day,
}

/// Chart types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ChartType {
    /// Instance-wide user statistics
    Users,
    /// Instance-wide note statistics
    Notes,
    /// Instance-wide drive statistics
    Drive,
    /// Federation statistics
    Federation,
    /// Per-user note statistics
    PerUserNotes,
    /// Per-user following statistics
    PerUserFollowing,
    /// Per-user drive statistics
    PerUserDrive,
}

/// Chart entry - time-series statistical data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartEntry {
    pub id: ChartEntryId,

    /// Chart type
    pub chart_type: ChartType,

    /// Time span (hour or day)
    pub span: ChartSpan,

    /// Date of the entry (for daily aggregation)
    pub date: Option<NaiveDate>,

    /// Hour of the entry (for hourly aggregation, 0-23)
    pub hour: Option<u8>,

    /// User ID (for per-user charts)
    pub user_id: Option<ActorId>,

    /// Total count at this point in time
    pub total: i64,

    /// Increase during this period
    pub inc: i64,

    /// Decrease during this period
    pub dec: i64,

    /// Additional data (JSON blob for flexibility)
    pub data: Option<serde_json::Value>,

    /// When this entry was created
    pub created_at: DateTime<Utc>,
}

impl ChartEntry {
    /// Create a new daily chart entry
    pub fn new_daily(
        chart_type: ChartType,
        date: NaiveDate,
        total: i64,
        inc: i64,
        dec: i64,
    ) -> Self {
        Self {
            id: ChartEntryId::new(),
            chart_type,
            span: ChartSpan::Day,
            date: Some(date),
            hour: None,
            user_id: None,
            total,
            inc,
            dec,
            data: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new hourly chart entry
    pub fn new_hourly(
        chart_type: ChartType,
        date: NaiveDate,
        hour: u8,
        total: i64,
        inc: i64,
        dec: i64,
    ) -> Self {
        Self {
            id: ChartEntryId::new(),
            chart_type,
            span: ChartSpan::Hour,
            date: Some(date),
            hour: Some(hour),
            user_id: None,
            total,
            inc,
            dec,
            data: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new per-user chart entry
    pub fn new_per_user(
        chart_type: ChartType,
        span: ChartSpan,
        user_id: ActorId,
        date: NaiveDate,
        total: i64,
        inc: i64,
        dec: i64,
    ) -> Self {
        Self {
            id: ChartEntryId::new(),
            chart_type,
            span,
            date: Some(date),
            hour: None,
            user_id: Some(user_id),
            total,
            inc,
            dec,
            data: None,
            created_at: Utc::now(),
        }
    }
}

/// Chart data response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartDataResponse {
    /// Labels (dates or hours)
    pub labels: Vec<String>,
    /// Total values
    pub total: Vec<i64>,
    /// Increase values
    pub inc: Vec<i64>,
    /// Decrease values
    pub dec: Vec<i64>,
}

/// Chart request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartQuery {
    /// Number of data points to return (default: 30 days)
    #[serde(default = "default_limit")]
    pub limit: u32,

    /// Time span (hour or day, default: day)
    #[serde(default)]
    pub span: ChartSpanQuery,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChartSpanQuery {
    #[default]
    Day,
    Hour,
}

impl From<ChartSpanQuery> for ChartSpan {
    fn from(query: ChartSpanQuery) -> Self {
        match query {
            ChartSpanQuery::Day => ChartSpan::Day,
            ChartSpanQuery::Hour => ChartSpan::Hour,
        }
    }
}

fn default_limit() -> u32 {
    30
}

/// Instance stats response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatsResponse {
    pub total_users: i64,
    pub active_users_month: i64,
    pub total_notes: i64,
    pub total_drive_usage: i64,
}

/// User stats response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatsResponse {
    pub notes_count: i64,
    pub following_count: i64,
    pub followers_count: i64,
    pub drive_usage: i64,
}
