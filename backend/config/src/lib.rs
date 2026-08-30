use serde::Deserialize;
use std::env;

/// アプリケーション設定
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub surrealdb_endpoint: String,
    pub surrealdb_namespace: String,
    pub surrealdb_database: String,
    pub surrealdb_username: String,
    pub surrealdb_password: String,

    pub surrealdb_pool_size: usize,

    pub dragonfly_url: String,

    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,

    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
    /// 信頼済みリバースプロキシ経由時のみ true。X-Forwarded-For をクライアント IP として使う。
    pub trust_proxy: bool,

    pub storage_type: String,
    pub local_storage_path: String,
    pub storage_s3_endpoint: Option<String>,
    pub storage_s3_bucket: Option<String>,
    pub storage_s3_access_key: Option<String>,
    pub storage_s3_secret_key: Option<String>,
    pub storage_s3_region: Option<String>,
    pub storage_s3_public_url: Option<String>,
    pub storage_gcs_bucket: Option<String>,
    pub storage_gcs_credentials: Option<String>,
    pub storage_gcs_public_url: Option<String>,

    pub instance_url: String,
    pub instance_name: String,

    /// VAPID private key (URL-safe base64, raw 32-byte EC key). When set, Web Push is enabled.
    pub vapid_private_key: Option<String>,
    /// Contact for VAPID `sub` claim (mailto: or https:)
    pub vapid_contact: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let instance_url =
            env::var("INSTANCE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        let jwt_secret = env::var("JWT_SECRET").map_err(|_| {
            anyhow::anyhow!("JWT_SECRET must be set (refuse to start with a default secret)")
        })?;
        if jwt_secret.is_empty() || jwt_secret == "change-me-in-production" {
            anyhow::bail!(
                "JWT_SECRET must be set to a non-default value (got empty or placeholder)"
            );
        }

        // 信頼済みリバースプロキシ経由時のみ true にする。ヘッダ偽装でレート制限を回避されないよう既定は false。
        let trust_proxy = env::var("TRUST_PROXY")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        Ok(Self {
            surrealdb_endpoint: env::var("SURREALDB_ENDPOINT")
                .unwrap_or_else(|_| "ws://localhost:8000".to_string()),
            surrealdb_namespace: env::var("SURREALDB_NAMESPACE")
                .unwrap_or_else(|_| "mithic".to_string()),
            surrealdb_database: env::var("SURREALDB_DATABASE")
                .unwrap_or_else(|_| "main".to_string()),
            surrealdb_username: env::var("SURREALDB_USERNAME")
                .unwrap_or_else(|_| "root".to_string()),
            surrealdb_password: env::var("SURREALDB_PASSWORD")
                .unwrap_or_else(|_| "root".to_string()),

            surrealdb_pool_size: env::var("SURREALDB_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(4),

            dragonfly_url: env::var("DRAGONFLY_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),

            jwt_secret,
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(24),

            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .map(|s| s.split(',').map(|s| s.to_string()).collect())
                .unwrap_or_else(|_| vec![instance_url.clone()]),
            trust_proxy,

            storage_type: env::var("STORAGE_TYPE").unwrap_or_else(|_| "local".to_string()),
            local_storage_path: env::var("LOCAL_STORAGE_PATH")
                .unwrap_or_else(|_| "./files".to_string()),
            storage_s3_endpoint: env::var("STORAGE_S3_ENDPOINT").ok(),
            storage_s3_bucket: env::var("STORAGE_S3_BUCKET").ok(),
            storage_s3_access_key: env::var("STORAGE_S3_ACCESS_KEY").ok(),
            storage_s3_secret_key: env::var("STORAGE_S3_SECRET_KEY").ok(),
            storage_s3_region: env::var("STORAGE_S3_REGION").ok(),
            storage_s3_public_url: env::var("STORAGE_S3_PUBLIC_URL").ok(),
            storage_gcs_bucket: env::var("STORAGE_GCS_BUCKET").ok(),
            storage_gcs_credentials: env::var("STORAGE_GCS_CREDENTIALS").ok(),
            storage_gcs_public_url: env::var("STORAGE_GCS_PUBLIC_URL").ok(),

            instance_url,
            instance_name: env::var("INSTANCE_NAME").unwrap_or_else(|_| "Mithic".to_string()),

            vapid_private_key: env::var("VAPID_PRIVATE_KEY")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            vapid_contact: env::var("VAPID_CONTACT")
                .unwrap_or_else(|_| "mailto:admin@localhost".to_string()),
        })
    }
}
