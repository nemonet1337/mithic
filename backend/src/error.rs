use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::i18n::{I18N, DEFAULT_LOCALE};
use crate::middleware::RequestLocale;

/// アプリケーションエラー型
#[derive(Debug)]
pub enum AppError {
    /// 認証エラー
    Unauthorized(String),
    /// 権限エラー
    Forbidden(String),
    /// リソースが見つからない
    NotFound(String),
    /// バリデーションエラー
    Validation(String),
    /// データベースエラー
    Database(surrealdb::Error),
    /// Redisエラー
    Redis(redis::RedisError),
    /// 内部サーバーエラー
    Internal(String),
}

impl AppError {
    /// エラーをレスポンスに変換（ロケール指定付き）
    pub fn into_response_with_locale(self, locale: &str) -> Response {
        // ログ記録
        match &self {
            AppError::Database(e) => tracing::error!("Database error: {}", e),
            AppError::Redis(e) => tracing::error!("Redis error: {}", e),
            AppError::Internal(msg) => tracing::error!("Internal error: {}", msg),
            _ => {}
        }

        let details = self.to_error_details();
        let status = self.status_code();

        // 指定されたロケールで翻訳
        let i18n = I18N::new();
        let error_message = i18n.translate_with_args(locale, details.key, None);

        // 詳細メッセージがある場合は追加
        let body = if let Some(detail) = details.detail {
            Json(json!({
                "error": true,
                "message": error_message,
                "detail": detail,
            }))
        } else {
            Json(json!({
                "error": true,
                "message": error_message,
            }))
        };

        (status, body).into_response()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // デフォルトの英語メッセージ
        let msg = match self {
            AppError::Unauthorized(msg) => format!("Unauthorized: {}", msg),
            AppError::Forbidden(msg) => format!("Forbidden: {}", msg),
            AppError::NotFound(msg) => format!("Not found: {}", msg),
            AppError::Validation(msg) => format!("Validation error: {}", msg),
            AppError::Database(e) => format!("Database error: {}", e),
            AppError::Redis(e) => format!("Redis error: {}", e),
            AppError::Internal(msg) => format!("Internal error: {}", msg),
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for AppError {}

impl From<surrealdb::Error> for AppError {
    fn from(e: surrealdb::Error) -> Self {
        AppError::Database(e)
    }
}

impl From<redis::RedisError> for AppError {
    fn from(e: redis::RedisError) -> Self {
        AppError::Redis(e)
    }
}

/// エラーキーと詳細メッセージを分離
#[derive(Debug)]
struct ErrorDetails {
    key: &'static str,
    detail: Option<String>,
}

impl AppError {
    /// エラーの翻訳キーと詳細を取得
    fn to_error_details(&self) -> ErrorDetails {
        match self {
            AppError::Unauthorized(_) => ErrorDetails {
                key: "error-unauthorized",
                detail: None,
            },
            AppError::Forbidden(_) => ErrorDetails {
                key: "error-forbidden",
                detail: None,
            },
            AppError::NotFound(_) => ErrorDetails {
                key: "error-not-found",
                detail: None,
            },
            AppError::Validation(msg) => ErrorDetails {
                key: "error-validation",
                detail: Some(msg.clone()),
            },
            AppError::Database(_) => ErrorDetails {
                key: "error-database",
                detail: None,
            },
            AppError::Redis(_) => ErrorDetails {
                key: "error-cache",
                detail: None,
            },
            AppError::Internal(_) => ErrorDetails {
                key: "error-internal",
                detail: None,
            },
        }
    }

    /// HTTPステータスコードを取得
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // デフォルトロケールで変換（Extensionが取得できない場合のフォールバック）
        self.into_response_with_locale(DEFAULT_LOCALE)
    }
}

/// Axumハンドラ用のエラーレスポンス生成ヘルパー
/// 
/// 使用例:
/// ```rust
/// pub async fn handler(
///     Extension(locale): Extension<RequestLocale>,
/// ) -> Result<Json<SomeType>, AppError> {
///     // ...
/// }
/// ```
pub fn error_response(err: AppError, locale: &RequestLocale) -> Response {
    err.into_response_with_locale(locale.as_str())
}

/// Result型のエイリアス
pub type Result<T> = std::result::Result<T, AppError>;
