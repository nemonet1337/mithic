use gloo_net::http::{Request, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub fn api_base() -> &'static str {
    "/api"
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub status:  u16,
    pub code:    String,
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.status, self.code, self.message)
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
        "GET"    => Request::get(&url),
        "POST"   => Request::post(&url),
        "PATCH"  => Request::patch(&url),
        "DELETE" => Request::delete(&url),
        "PUT"    => Request::put(&url),
        other    => panic!("unsupported HTTP method: {other}"),
    };

    if let Some(tok) = token {
        req = req.header("Authorization", &format!("Bearer {tok}"));
    }

    let response: Response = if let Some(b) = body {
        req.json(b)
            .map_err(|e| ApiError { status: 0, code: "serialize".into(), message: e.to_string() })?
            .send()
            .await
    } else {
        req.send().await
    }
    .map_err(|e| ApiError { status: 0, code: "network".into(), message: e.to_string() })?;

    let status = response.status();

    if status == 429 {
        return Err(ApiError { status, code: "rate_limit".into(), message: "レートリミット".into() });
    }

    if !response.ok() {
        let err: ApiError = response.json().await.unwrap_or(ApiError {
            status,
            code: "unknown".into(),
            message: format!("HTTP {status}"),
        });
        return Err(err);
    }

    response
        .json::<T>()
        .await
        .map_err(|e| ApiError { status: 0, code: "deserialize".into(), message: e.to_string() })
}

pub async fn request_with_retry<T, B>(
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<&B>,
    max_attempts: u32,
) -> Result<T, ApiError>
where
    T: DeserializeOwned,
    B: Serialize,
{
    let mut delay_ms = 3_000u32;
    for attempt in 0..max_attempts {
        match request(method, path, token, body).await {
            Ok(v) => return Ok(v),
            Err(e) if e.status == 429 && attempt + 1 < max_attempts => {
                gloo_timers::future::sleep(std::time::Duration::from_millis(delay_ms.into())).await;
                delay_ms *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
