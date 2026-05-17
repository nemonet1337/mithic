use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::actor::ActorId;

pub type ChartEntryId = Ulid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChartSpan { Hour, Day }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ChartType { Users, Notes, Drive, Federation, PerUserNotes, PerUserFollowing, PerUserDrive }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartEntry {
    pub id: ChartEntryId,
    pub chart_type: ChartType,
    pub span: ChartSpan,
    pub date: Option<NaiveDate>,
    pub hour: Option<u8>,
    pub user_id: Option<ActorId>,
    pub total: i64,
    pub inc: i64,
    pub dec: i64,
    pub data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl ChartEntry {
    pub fn new_daily(chart_type: ChartType, date: NaiveDate, total: i64, inc: i64, dec: i64) -> Self {
        Self { id: ChartEntryId::new(), chart_type, span: ChartSpan::Day, date: Some(date), hour: None, user_id: None, total, inc, dec, data: None, created_at: Utc::now() }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartDataResponse {
    pub labels: Vec<String>,
    pub total: Vec<i64>,
    pub inc: Vec<i64>,
    pub dec: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartQuery {
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub span: ChartSpanQuery,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChartSpanQuery { #[default] Day, Hour }

impl From<ChartSpanQuery> for ChartSpan {
    fn from(q: ChartSpanQuery) -> Self {
        match q { ChartSpanQuery::Day => ChartSpan::Day, ChartSpanQuery::Hour => ChartSpan::Hour }
    }
}

fn default_limit() -> u32 { 30 }

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatsResponse {
    pub total_users: i64,
    pub active_users_month: i64,
    pub total_notes: i64,
    pub total_drive_usage: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStatsResponse {
    pub notes_count: i64,
    pub following_count: i64,
    pub followers_count: i64,
    pub drive_usage: i64,
}
