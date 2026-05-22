use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::prelude::*;
use serde_json::json;
use std::time::Duration;
use tracing::{error, info, warn};

use mithic_core::AppError;

use crate::state::AppState;

#[derive(Debug)]
pub enum SignatureError {
    MissingHeader(String),
    InvalidFormat,
    VerificationFailed,
    DigestMismatch,
    UnsupportedAlgorithm,
    ActorKeyNotFound,
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            SignatureError::MissingHeader(h) => format!("Missing required header: {}", h),
            SignatureError::InvalidFormat => "Invalid signature format".to_string(),
            SignatureError::VerificationFailed => "Signature verification failed".to_string(),
            SignatureError::DigestMismatch => "Digest verification failed".to_string(),
            SignatureError::UnsupportedAlgorithm => "Unsupported algorithm".to_string(),
            SignatureError::ActorKeyNotFound => "Actor key not found".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for SignatureError {}

impl IntoResponse for SignatureError {
    fn into_response(self) -> Response {
        let status = match &self {
            SignatureError::MissingHeader(_) => StatusCode::BAD_REQUEST,
            SignatureError::InvalidFormat => StatusCode::BAD_REQUEST,
            SignatureError::VerificationFailed => StatusCode::UNAUTHORIZED,
            SignatureError::DigestMismatch => StatusCode::UNAUTHORIZED,
            SignatureError::UnsupportedAlgorithm => StatusCode::BAD_REQUEST,
            SignatureError::ActorKeyNotFound => StatusCode::NOT_FOUND,
        };
        let body = json!({ "error": "http_signature_error", "message": self.to_string() });
        (status, axum::Json(body)).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct HttpSignature {
    pub key_id: String,
    pub signature: String,
    pub headers: Vec<String>,
    pub algorithm: String,
}

impl HttpSignature {
    pub fn parse(header_value: &str) -> Result<Self, SignatureError> {
        let mut key_id = None;
        let mut signature = None;
        let mut headers = vec![
            "(request-target)".to_string(),
            "host".to_string(),
            "date".to_string(),
        ];
        let mut algorithm = "rsa-sha256".to_string();

        for part in header_value.split(',') {
            let part = part.trim();
            if let Some(value) = part.strip_prefix("keyId=\"") {
                key_id = Some(value.trim_end_matches('"').to_string());
            } else if let Some(value) = part.strip_prefix("signature=\"") {
                signature = Some(value.trim_end_matches('"').to_string());
            } else if let Some(value) = part.strip_prefix("headers=\"") {
                headers = value
                    .trim_end_matches('"')
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            } else if let Some(value) = part.strip_prefix("algorithm=\"") {
                algorithm = value.trim_end_matches('"').to_string();
            }
        }

        Ok(Self {
            key_id: key_id.ok_or(SignatureError::InvalidFormat)?,
            signature: signature.ok_or(SignatureError::InvalidFormat)?,
            headers,
            algorithm,
        })
    }
}

pub async fn verify_http_signature(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, SignatureError> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| SignatureError::MissingHeader("Host".to_string()))?;

    let signature_header = headers
        .get("signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| SignatureError::MissingHeader("Signature".to_string()))?;

    let date_header = headers
        .get("date")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| SignatureError::MissingHeader("Date".to_string()))?;

    let request_time = chrono::DateTime::parse_from_rfc2822(date_header)
        .map_err(|_| SignatureError::InvalidFormat)?;
    let now = chrono::Utc::now();
    let time_diff = (now - request_time.with_timezone(&chrono::Utc))
        .num_seconds()
        .abs();
    if time_diff > 30 * 60 {
        warn!(
            "Request time too old or in future: {} (diff: {}s)",
            date_header, time_diff
        );
        return Err(SignatureError::VerificationFailed);
    }

    let digest_header = headers.get("digest").and_then(|v| v.to_str().ok());
    let signature = HttpSignature::parse(signature_header)?;
    info!("Verifying HTTP signature from key_id: {}", signature.key_id);

    let public_key_pem = fetch_actor_public_key(&state, &signature.key_id)
        .await
        .map_err(|e| {
            warn!("Failed to fetch public key for {}: {}", signature.key_id, e);
            SignatureError::ActorKeyNotFound
        })?;

    let (parts, body) = request.into_parts();
    let body_bytes = if method == axum::http::Method::POST {
        to_bytes(body, usize::MAX)
            .await
            .map_err(|_| SignatureError::DigestMismatch)?
    } else {
        axum::body::Bytes::new()
    };

    if method == axum::http::Method::POST {
        let digest =
            digest_header.ok_or_else(|| SignatureError::MissingHeader("Digest".to_string()))?;
        if !verify_digest(&body_bytes, digest) {
            warn!("Digest verification failed");
            return Err(SignatureError::DigestMismatch);
        }
    }

    let request_target = format!("{} {}", method.as_str().to_lowercase(), uri.path());
    if !verify_signature(
        &public_key_pem,
        &signature,
        &request_target,
        host,
        date_header,
        &headers,
    )? {
        warn!("HTTP signature verification failed");
        return Err(SignatureError::VerificationFailed);
    }

    info!("HTTP signature verified successfully");
    let request = Request::from_parts(parts, Body::from(body_bytes));
    Ok(next.run(request).await)
}

fn verify_signature(
    _public_key_pem: &str,
    signature: &HttpSignature,
    _request_target: &str,
    _host: &str,
    _date: &str,
    _headers: &axum::http::HeaderMap,
) -> Result<bool, SignatureError> {
    if signature.algorithm != "rsa-sha256" && signature.algorithm != "hs2019" {
        warn!("Unsupported algorithm: {}", signature.algorithm);
        return Err(SignatureError::UnsupportedAlgorithm);
    }
    // TODO: implement RSA-SHA256 verification (requires openssl or ring)
    warn!("HTTP signature cryptographic verification is not yet implemented");
    Ok(true)
}

fn build_signing_string(
    headers: &[String],
    request_target: &str,
    host: &str,
    date: &str,
    _all_headers: &axum::http::HeaderMap,
) -> String {
    let mut parts = Vec::new();
    for header in headers {
        let value = match header.as_str() {
            "(request-target)" => request_target.to_string(),
            "host" => host.to_string(),
            "date" => date.to_string(),
            _ => continue,
        };
        parts.push(format!("{}: {}", header, value));
    }
    parts.join("\n")
}

fn verify_digest(body: &[u8], digest_header: &str) -> bool {
    let parts: Vec<&str> = digest_header.splitn(2, '=').collect();
    if parts.len() != 2 {
        return false;
    }
    if !parts[0].eq_ignore_ascii_case("sha-256") && !parts[0].eq_ignore_ascii_case("SHA-256") {
        return false;
    }

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(body);
    let actual_digest = BASE64_STANDARD.encode(hasher.finalize());
    actual_digest == parts[1]
}

async fn fetch_actor_public_key(state: &AppState, key_id: &str) -> Result<String, AppError> {
    use redis::AsyncCommands;

    let cache_key = format!("actor_key:{}", key_id);
    let mut redis = state.dragonfly().clone();
    let cached: Option<String> = redis.get(&cache_key).await.ok();
    if let Some(public_key_pem) = cached {
        info!("Cache hit for public key: {}", key_id);
        return Ok(public_key_pem);
    }

    let actor_url = key_id.split('#').next().unwrap_or(key_id);
    info!("Fetching public key for actor: {}", actor_url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get(actor_url)
        .header("Accept", "application/activity+json, application/ld+json")
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch actor: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Internal(format!(
            "Actor fetch failed with status: {}",
            response.status()
        )));
    }

    let actor_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse actor JSON: {}", e)))?;

    let public_key_pem = actor_data
        .get("publicKey")
        .and_then(|pk| pk.get("publicKeyPem"))
        .and_then(|pem| pem.as_str())
        .ok_or_else(|| AppError::Internal("Public key not found in actor data".to_string()))?;

    let pem_string = public_key_pem.to_string();
    let _: Result<(), _> = redis.set_ex(&cache_key, &pem_string, 24 * 60 * 60).await;
    info!("Successfully fetched and cached public key for: {}", key_id);
    Ok(pem_string)
}
