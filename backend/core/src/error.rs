use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use mithic_i18n::{DEFAULT_LOCALE, I18N};

/// アプリケーションエラー型
#[derive(Debug)]
pub enum AppError {
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Validation(String),
    Database(surrealdb::Error),
    Redis(redis::RedisError),
    Internal(String),
}

impl AppError {
    /// エラーをロケール指定付きでレスポンスに変換
    pub fn into_response_with_locale(self, locale: &str) -> Response {
        match &self {
            AppError::Database(e) => tracing::error!("Database error: {}", e),
            AppError::Redis(e) => tracing::error!("Redis error: {}", e),
            AppError::Internal(msg) => tracing::error!("Internal error: {}", msg),
            _ => {}
        }

        let details = self.to_error_details();
        let status = self.status_code();

        let error_message = I18N.translate_with_args(locale, details.key, None);

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

#[derive(Debug)]
struct ErrorDetails {
    key: &'static str,
    detail: Option<String>,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.into_response_with_locale(DEFAULT_LOCALE)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
