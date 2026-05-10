use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use std::env;

/// アプリケーション設定
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    // SurrealDB設定
    pub surrealdb_endpoint: String,
    pub surrealdb_namespace: String,
    pub surrealdb_database: String,
    pub surrealdb_username: String,
    pub surrealdb_password: String,

    // Dragonfly (Redis互換) 設定
    pub dragonfly_url: String,

    // JWT設定
    pub jwt_secret: Secret<String>,
    pub jwt_expiry_hours: i64,

    // サーバー設定
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,

    // ファイルストレージ設定
    pub storage_type: String, // "local" or "s3"
    pub local_storage_path: String,

    // ActivityPub設定
    pub instance_url: String,
    pub instance_name: String,
}

impl AppConfig {
    /// 環境変数から設定を読み込み
    pub fn from_env() -> anyhow::Result<Self> {
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

            dragonfly_url: env::var("DRAGONFLY_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),

            jwt_secret: Secret::new(
                env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "change-me-in-production".to_string())
            ),
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
                .unwrap_or_else(|_| vec!["*".to_string()]),

            storage_type: env::var("STORAGE_TYPE").unwrap_or_else(|_| "local".to_string()),
            local_storage_path: env::var("LOCAL_STORAGE_PATH")
                .unwrap_or_else(|_| "./files".to_string()),

            instance_url: env::var("INSTANCE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            instance_name: env::var("INSTANCE_NAME")
                .unwrap_or_else(|_| "Mithic".to_string()),
        })
    }

    /// JWTシークレットを取得
    pub fn jwt_secret(&self) -> &str {
        self.jwt_secret.expose_secret()
    }
}
