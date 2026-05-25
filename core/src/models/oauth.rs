use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

pub type OAuthAppId = Ulid;
pub type OAuthTokenId = Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct OAuthApp {
    pub id: OAuthAppId,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    #[validate(length(min = 1, max = 256))]
    pub client_name: String,
    #[validate(url)]
    pub redirect_uris: String,
    pub client_id: String,
    pub client_secret: String,
    #[validate(url)]
    pub website: Option<String>,
    pub scopes: Vec<String>,
    pub user_id: Option<Ulid>,
    pub is_confidential: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthToken {
    pub id: OAuthTokenId,
    pub created_at: DateTime<Utc>,
    pub app_id: OAuthAppId,
    pub user_id: Ulid,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateOAuthAppRequest {
    #[validate(length(min = 1, max = 256))]
    pub client_name: String,
    #[validate(url)]
    pub redirect_uris: String,
    #[validate(url)]
    pub website: Option<String>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthAppResponse {
    pub id: OAuthAppId,
    pub client_id: String,
    pub client_secret: String,
    pub name: String,
    pub redirect_uris: String,
    pub website: Option<String>,
    pub scopes: Vec<String>,
}

impl From<OAuthApp> for OAuthAppResponse {
    fn from(app: OAuthApp) -> Self {
        Self {
            id: app.id,
            client_id: app.client_id,
            client_secret: app.client_secret,
            name: app.client_name,
            redirect_uris: app.redirect_uris,
            website: app.website,
            scopes: app.scopes,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
    pub created_at: i64,
}
