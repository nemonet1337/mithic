use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

#[derive(Debug)]
pub enum ClientError {
    InvalidHeader(reqwest::header::InvalidHeaderValue),
    Build(reqwest::Error),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHeader(error) => write!(f, "invalid auth header: {error}"),
            Self::Build(error) => write!(f, "failed to build HTTP client: {error}"),
        }
    }
}

impl std::error::Error for ClientError {}

pub fn api_base() -> String {
    option_env!("MITHIC_API_BASE")
        .unwrap_or("/api")
        .trim_end_matches('/')
        .to_string()
}

pub fn authed_client(token: &str) -> Result<reqwest::Client, ClientError> {
    let mut headers = HeaderMap::new();
    let value =
        HeaderValue::from_str(&format!("Bearer {token}")).map_err(ClientError::InvalidHeader)?;
    headers.insert(AUTHORIZATION, value);
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(ClientError::Build)
}
