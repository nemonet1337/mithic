//! Chart service for statistics
//!
//! Provides time-series statistics for instance and users.

use chrono::{Datelike, Duration, NaiveDate, Timelike, Utc};
use tracing::{error, info};

use crate::{
    db::{DragonflyClient, SurrealClient},
    error::{AppError, Result},
    models::{
        ActorId, ChartDataResponse, ChartEntry, ChartEntryId, ChartQuery, ChartSpan, ChartType,
        InstanceStatsResponse, UserStatsResponse,
    },
};

/// Chart service for managing statistics
#[derive(Debug, Clone)]
pub struct ChartService {
    surreal: SurrealClient,
    dragonfly: DragonflyClient,
}

impl ChartService {
    /// Create a new chart service
    pub fn new(surreal: SurrealClient, dragonfly: DragonflyClient) -> Self {
        Self {
            surreal,
            dragonfly,
        }
    }

    /// Get instance stats (current snapshot)
    pub async fn get_instance_stats(&self) -> Result<InstanceStatsResponse> {
        // Query total users
        let total_users: i64 = self
            .surreal
            .query("SELECT count() FROM user WHERE is_bot = false GROUP BY all")
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        // Query active users (last 30 days)
        let active_since = Utc::now() - Duration::days(30);
        let active_users_month: i64 = self
            .surreal
            .query("SELECT count() FROM user WHERE last_active_at > $active_since AND is_bot = false GROUP BY all")
            .bind(("active_since", active_since))
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        // Query total notes
        let total_notes: i64 = self
            .surreal
            .query("SELECT count() FROM note GROUP BY all")
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        // Query total drive usage
        let total_drive_usage: i64 = self
            .surreal
            .query("SELECT math::sum(size) FROM drive_file GROUP BY all")
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        Ok(InstanceStatsResponse {
            total_users,
            active_users_month,
            total_notes,
            total_drive_usage,
        })
    }

    /// Get user stats
    pub async fn get_user_stats(&self, user_id: &ActorId) -> Result<UserStatsResponse> {
        // Query notes count
        let notes_count: i64 = self
            .surreal
            .query("SELECT count() FROM note WHERE author_id = $user_id GROUP BY all")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        // Query following count
        let following_count: i64 = self
            .surreal
            .query("SELECT count() FROM follow WHERE follower_id = $user_id GROUP BY all")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        // Query followers count
        let followers_count: i64 = self
            .surreal
            .query("SELECT count() FROM follow WHERE followee_id = $user_id GROUP BY all")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        // Query drive usage
        let drive_usage: i64 = self
            .surreal
            .query("SELECT math::sum(size) FROM drive_file WHERE owner_id = $user_id GROUP BY all")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        Ok(UserStatsResponse {
            notes_count,
            following_count,
            followers_count,
            drive_usage,
        })
    }

    /// Get chart data for a specific chart type
    pub async fn get_chart_data(
        &self,
        chart_type: ChartType,
        span: ChartSpan,
        limit: u32,
    ) -> Result<ChartDataResponse> {
        let entries: Vec<ChartEntry> = self
            .surreal
            .query("SELECT * FROM chart_entry WHERE chart_type = $chart_type AND span = $span ORDER BY date DESC, hour DESC LIMIT $limit")
            .bind(("chart_type", format!("{:?}", chart_type).to_lowercase()))
            .bind(("span", format!("{:?}", span).to_lowercase()))
            .bind(("limit", limit as i64))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        let mut labels = Vec::new();
        let mut total = Vec::new();
        let mut inc = Vec::new();
        let mut dec = Vec::new();

        for entry in entries.iter().rev() {
            let label = match span {
                ChartSpan::Day => entry.date.map(|d| d.to_string()).unwrap_or_default(),
                ChartSpan::Hour => format!(
                    "{} {:02}:00",
                    entry.date.map(|d| d.to_string()).unwrap_or_default(),
                    entry.hour.unwrap_or(0)
                ),
            };
            labels.push(label);
            total.push(entry.total);
            inc.push(entry.inc);
            dec.push(entry.dec);
        }

        Ok(ChartDataResponse {
            labels,
            total,
            inc,
            dec,
        })
    }

    /// Get per-user chart data
    pub async fn get_user_chart_data(
        &self,
        chart_type: ChartType,
        user_id: &ActorId,
        span: ChartSpan,
        limit: u32,
    ) -> Result<ChartDataResponse> {
        let entries: Vec<ChartEntry> = self
            .surreal
            .query("SELECT * FROM chart_entry WHERE chart_type = $chart_type AND user_id = $user_id AND span = $span ORDER BY date DESC, hour DESC LIMIT $limit")
            .bind(("chart_type", format!("{:?}", chart_type).to_lowercase()))
            .bind(("user_id", user_id.to_string()))
            .bind(("span", format!("{:?}", span).to_lowercase()))
            .bind(("limit", limit as i64))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        let mut labels = Vec::new();
        let mut total = Vec::new();
        let mut inc = Vec::new();
        let mut dec = Vec::new();

        for entry in entries.iter().rev() {
            let label = match span {
                ChartSpan::Day => entry.date.map(|d| d.to_string()).unwrap_or_default(),
                ChartSpan::Hour => format!(
                    "{} {:02}:00",
                    entry.date.map(|d| d.to_string()).unwrap_or_default(),
                    entry.hour.unwrap_or(0)
                ),
            };
            labels.push(label);
            total.push(entry.total);
            inc.push(entry.inc);
            dec.push(entry.dec);
        }

        Ok(ChartDataResponse {
            labels,
            total,
            inc,
            dec,
        })
    }

    /// Record a chart entry (called by background jobs)
    pub async fn record_entry(&self, entry: ChartEntry) -> Result<()> {
        self.surreal
            .create::<Option<ChartEntry>>(
                ("chart_entry", entry.id.to_string()),
            )
            .content(entry)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    /// Aggregate daily statistics (should be called by a scheduled job)
    pub async fn aggregate_daily(&self, date: NaiveDate) -> Result<()> {
        let yesterday = date - Duration::days(1);

        // Users chart
        let total_users = self
            .surreal
            .query("SELECT count() FROM user WHERE is_bot = false GROUP BY all")
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        let new_users: i64 = self
            .surreal
            .query("SELECT count() FROM user WHERE created_at > $yesterday AND created_at <= $today GROUP BY all")
            .bind(("yesterday", yesterday))
            .bind(("today", date))
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        let entry = ChartEntry::new_daily(ChartType::Users, date, total_users, new_users, 0);
        self.record_entry(entry).await?;

        // Notes chart
        let total_notes = self
            .surreal
            .query("SELECT count() FROM note GROUP BY all")
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        let new_notes: i64 = self
            .surreal
            .query("SELECT count() FROM note WHERE created_at > $yesterday AND created_at <= $today GROUP BY all")
            .bind(("yesterday", yesterday))
            .bind(("today", date))
            .await
            .and_then(|mut res| res.take::<Option<i64>>(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or(0);

        let entry = ChartEntry::new_daily(ChartType::Notes, date, total_notes, new_notes, 0);
        self.record_entry(entry).await?;

        info!("Aggregated daily stats for {}", date);

        Ok(())
    }
}
