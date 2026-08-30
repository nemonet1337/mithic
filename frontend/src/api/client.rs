use gloo_net::http::{Request, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub fn api_base() -> &'static str {
    "/api/v1"
}

/// Minimal percent-encoding for query path segments (ASCII unreserved left as-is).
pub fn urlencoding_loose(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    #[serde(default)]
    pub status: u16,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub message: String,
    /// バックエンドが付ける具体メッセージ (例: Invalid username or password)
    #[serde(default)]
    pub detail: Option<String>,
}

impl ApiError {
    /// ユーザー表示用メッセージ (detail があれば優先)
    pub fn user_message(&self) -> String {
        self.detail
            .clone()
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| self.message.clone())
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.status,
            self.code,
            self.user_message()
        )
    }
}

impl std::error::Error for ApiError {}

pub async fn request<T, B>(
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<&B>,
) -> Result<T, ApiError>
where
    T: DeserializeOwned,
    B: Serialize,
{
    let url = format!("{}/{}", api_base(), path.trim_start_matches('/'));

    let mut req = match method {
        "GET" => Request::get(&url),
        "POST" => Request::post(&url),
        "PATCH" => Request::patch(&url),
        "DELETE" => Request::delete(&url),
        "PUT" => Request::put(&url),
        other => panic!("unsupported HTTP method: {other}"),
    };

    if let Some(tok) = token {
        req = req.header("Authorization", &format!("Bearer {tok}"));
    }

    let response: Response = if let Some(b) = body {
        req.json(b)
            .map_err(|e| ApiError {
                status: 0,
                code: "serialize".into(),
                message: e.to_string(),
                detail: None,
            })?
            .send()
            .await
    } else {
        req.send().await
    }
    .map_err(|e| {
        let raw = e.to_string();
        // Failed to fetch 等はユーザー向けに分かりやすい文言へ
        let message = if raw.contains("Failed to fetch")
            || raw.contains("NetworkError")
            || raw.contains("Load failed")
        {
            "サーバーに接続できません。バックエンドが起動しているか確認してください。".into()
        } else {
            raw
        };
        ApiError {
            status: 0,
            code: "network".into(),
            message,
            detail: None,
        }
    })?;

    let status = response.status();

    if status == 429 {
        return Err(ApiError {
            status,
            code: "rate_limit".into(),
            message: "レートリミット".into(),
            detail: None,
        });
    }

    if !response.ok() {
        // バックエンドは { error, message, detail? } 形式。status/code は無いので default で受ける。
        let mut err: ApiError = response.json().await.unwrap_or(ApiError {
            status,
            code: "unknown".into(),
            message: format!("HTTP {status}"),
            detail: None,
        });
        if err.status == 0 {
            err.status = status;
        }
        if err.message.is_empty() {
            err.message = format!("HTTP {status}");
        }
        return Err(err);
    }

    response.json::<T>().await.map_err(|e| ApiError {
        status: 0,
        code: "deserialize".into(),
        message: e.to_string(),
        detail: None,
    })
}
