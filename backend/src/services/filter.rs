//! Filter service
//!
//! Content filtering (word mute, regex filters).

use tracing::info;

use crate::{
    db::{DragonflyClient, SurrealClient},
    error::{AppError, Result},
    models::{CreateFilterRequest, Filter, FilterId, UpdateFilterRequest, Ulid},
};

/// Filter service
#[derive(Debug, Clone)]
pub struct FilterService {
    surreal: SurrealClient,
    dragonfly: DragonflyClient,
}

impl FilterService {
    pub fn new(surreal: SurrealClient, dragonfly: DragonflyClient) -> Self {
        Self {
            surreal,
            dragonfly,
        }
    }

    /// Create filter
    pub async fn create_filter(&self, user_id: Ulid, req: CreateFilterRequest) -> Result<Filter> {
        let expires_at = req.expires_in.map(|seconds| {
            chrono::Utc::now() + chrono::Duration::seconds(seconds)
        });

        let filter = Filter {
            id: Ulid::new(),
            created_at: chrono::Utc::now(),
            updated_at: None,
            user_id,
            phrase: req.phrase,
            is_regex: req.is_regex.unwrap_or(false),
            hide_completely: req.hide_completely.unwrap_or(true),
            expires_at,
            context: req.context.unwrap_or_else(|| vec!["home".to_string(), "notifications".to_string()]),
        };

        self.surreal
            .create::<Option<Filter>>(("filter", filter.id.to_string()))
            .content(filter.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Created filter {} for user {}", filter.id, user_id);

        Ok(filter)
    }

    /// Get user's filters
    pub async fn get_filters(&self, user_id: Ulid) -> Result<Vec<Filter>> {
        let filters: Vec<Filter> = self
            .surreal
            .query("SELECT * FROM filter WHERE user_id = $user_id ORDER BY created_at DESC")
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        // Filter out expired filters
        let now = chrono::Utc::now();
        let active_filters: Vec<Filter> = filters
            .into_iter()
            .filter(|f| {
                f.expires_at
                    .map(|exp| exp > now)
                    .unwrap_or(true)
            })
            .collect();

        Ok(active_filters)
    }

    /// Get filter by ID
    pub async fn get_filter(&self, filter_id: FilterId, user_id: Ulid) -> Result<Filter> {
        let filter: Option<Filter> = self
            .surreal
            .query("SELECT * FROM filter WHERE id = $id AND user_id = $user_id LIMIT 1")
            .bind(("id", filter_id.to_string()))
            .bind(("user_id", user_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?;

        filter.ok_or_else(|| AppError::NotFound("Filter not found".to_string()))
    }

    /// Update filter
    pub async fn update_filter(&self, filter_id: FilterId, user_id: Ulid, req: UpdateFilterRequest) -> Result<Filter> {
        let mut updates = Vec::new();

        if let Some(phrase) = req.phrase {
            updates.push(format!("phrase = '{}'", phrase.replace("'", "''")));
        }

        if let Some(is_regex) = req.is_regex {
            updates.push(format!("is_regex = {}", is_regex));
        }

        if let Some(hide_completely) = req.hide_completely {
            updates.push(format!("hide_completely = {}", hide_completely));
        }

        if let Some(expires_in) = req.expires_in {
            let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in);
            updates.push(format!("expires_at = '{}'", expires_at.to_rfc3339()));
        }

        if let Some(context) = req.context {
            let context_str = format!("[{}]", context.iter().map(|c| format!("'{}'", c.replace("'", "''"))).collect::<Vec<_>>().join(", "));
            updates.push(format!("context = {}", context_str));
        }

        if !updates.is_empty() {
            updates.push("updated_at = time::now()".to_string());
            let query = format!(
                "UPDATE filter:{} SET {}",
                filter_id,
                updates.join(", ")
            );

            self.surreal
                .query(&query)
                .await
                .map_err(|e| AppError::Database(e))?;

            info!("Updated filter {} for user {}", filter_id, user_id);
        }

        self.get_filter(filter_id, user_id).await
    }

    /// Delete filter
    pub async fn delete_filter(&self, filter_id: FilterId, user_id: Ulid) -> Result<()> {
        let filter = self.get_filter(filter_id, user_id).await?;

        self.surreal
            .delete(("filter", filter_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Deleted filter {} for user {}", filter_id, user_id);

        Ok(())
    }

    /// Check if content matches any filter
    pub async fn content_matches_filters(&self, user_id: Ulid, content: &str, context: &str) -> bool {
        let filters = match self.get_filters(user_id).await {
            Ok(f) => f,
            Err(_) => return false,
        };

        for filter in filters {
            if !filter.context.contains(&context.to_string()) {
                continue;
            }

            let matches = if filter.is_regex {
                // Try regex matching
                match regex::Regex::new(&filter.phrase) {
                    Ok(re) => re.is_match(content),
                    Err(_) => continue,
                }
            } else {
                // Simple substring match
                content.to_lowercase().contains(&filter.phrase.to_lowercase())
            };

            if matches {
                return true;
            }
        }

        false
    }
}
