//! OAuth application service
//!
//! OAuth 2.0 application and token management.

use tracing::info;

use crate::{
    db::{DragonflyClient, SurrealClient},
    error::{AppError, Result},
    models::{ActorId, CreateOAuthAppRequest, OAuthApp, OAuthAppId, OAuthToken, OAuthTokenId, Ulid},
};

/// OAuth service
#[derive(Debug, Clone)]
pub struct OAuthService {
    surreal: SurrealClient,
    dragonfly: DragonflyClient,
}

impl OAuthService {
    pub fn new(surreal: SurrealClient, dragonfly: DragonflyClient) -> Self {
        Self {
            surreal,
            dragonfly,
        }
    }

    /// Create OAuth application
    pub async fn create_app(
        &self,
        req: CreateOAuthAppRequest,
        user_id: Option<ActorId>,
    ) -> Result<OAuthApp> {
        let client_id = ulid::Ulid::new().to_string();
        let client_secret = ulid::Ulid::new().to_string();

        let app = OAuthApp {
            id: Ulid::new(),
            created_at: chrono::Utc::now(),
            updated_at: None,
            client_name: req.client_name,
            redirect_uris: req.redirect_uris,
            client_id,
            client_secret,
            website: req.website,
            scopes: req.scopes.unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]),
            user_id,
            is_confidential: true,
        };

        self.surreal
            .create::<Option<OAuthApp>>(("oauth_app", app.id.to_string()))
            .content(app.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Created OAuth app {}", app.id);

        Ok(app)
    }

    /// Get OAuth app by client ID
    pub async fn get_app_by_client_id(&self, client_id: &str) -> Result<Option<OAuthApp>> {
        let app: Option<OAuthApp> = self
            .surreal
            .query("SELECT * FROM oauth_app WHERE client_id = $client_id LIMIT 1")
            .bind(("client_id", client_id))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?;

        Ok(app)
    }

    /// Create OAuth token
    pub async fn create_token(
        &self,
        app_id: OAuthAppId,
        user_id: ActorId,
        scopes: Vec<String>,
    ) -> Result<OAuthToken> {
        let access_token = format!("{}_{}", ulid::Ulid::new(), ulid::Ulid::new());

        let token = OAuthToken {
            id: Ulid::new(),
            created_at: chrono::Utc::now(),
            app_id,
            user_id,
            access_token: access_token.clone(),
            refresh_token: None,
            scopes,
            expires_at: None,
        };

        self.surreal
            .create::<Option<OAuthToken>>(("oauth_token", token.id.to_string()))
            .content(token.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Created OAuth token for user {}", user_id);

        Ok(token)
    }

    /// Get OAuth token by access token
    pub async fn get_token_by_access_token(&self, access_token: &str) -> Result<Option<OAuthToken>> {
        let token: Option<OAuthToken> = self
            .surreal
            .query("SELECT * FROM oauth_token WHERE access_token = $access_token LIMIT 1")
            .bind(("access_token", access_token))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?;

        Ok(token)
    }

    /// Revoke OAuth token
    pub async fn revoke_token(&self, token_id: OAuthTokenId) -> Result<()> {
        self.surreal
            .delete(("oauth_token", token_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Revoked OAuth token {}", token_id);

        Ok(())
    }
}
