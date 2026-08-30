//! Axum middleware: verify inbound ActivityPub HTTP Signatures.
//! Crypto lives in `crate::federation::http_sig`.

use crate::federation::http_sig::{self, HttpSigError, HttpSignature};
use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::time::Duration;
use tracing::{info, warn};

use crate::AppError;

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
            SignatureError::MissingHeader(h) => format!("Missing required header: {h}"),
            SignatureError::InvalidFormat => "Invalid signature format".to_string(),
            SignatureError::VerificationFailed => "Signature verification failed".to_string(),
            SignatureError::DigestMismatch => "Digest verification failed".to_string(),
            SignatureError::UnsupportedAlgorithm => "Unsupported algorithm".to_string(),
            SignatureError::ActorKeyNotFound => "Actor key not found".to_string(),
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for SignatureError {}

impl From<HttpSigError> for SignatureError {
    fn from(e: HttpSigError) -> Self {
        match e {
            HttpSigError::InvalidFormat => Self::InvalidFormat,
            HttpSigError::UnsupportedAlgorithm => Self::UnsupportedAlgorithm,
        }
    }
}

impl IntoResponse for SignatureError {
    fn into_response(self) -> Response {
        let status = match &self {
            SignatureError::MissingHeader(_) | SignatureError::InvalidFormat => {
                StatusCode::BAD_REQUEST
            }
            SignatureError::UnsupportedAlgorithm => StatusCode::BAD_REQUEST,
            SignatureError::VerificationFailed | SignatureError::DigestMismatch => {
                StatusCode::UNAUTHORIZED
            }
            SignatureError::ActorKeyNotFound => StatusCode::NOT_FOUND,
        };
        let body = json!({ "error": "http_signature_error", "message": self.to_string() });
        (status, axum::Json(body)).into_response()
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

    if method == axum::http::Method::POST {
        let required = ["(request-target)", "host", "date", "digest"];
        for req in required {
            if !signature
                .headers
                .iter()
                .any(|h| h.eq_ignore_ascii_case(req))
            {
                warn!("Signature missing required header binding: {req}");
                return Err(SignatureError::VerificationFailed);
            }
        }
    }

    const MAX_BODY_BYTES: usize = 1024 * 1024; // 1 MiB
    let (parts, body) = request.into_parts();
    let body_bytes = if method == axum::http::Method::POST {
        to_bytes(body, MAX_BODY_BYTES)
            .await
            .map_err(|_| SignatureError::DigestMismatch)?
    } else {
        axum::body::Bytes::new()
    };

    if method == axum::http::Method::POST {
        let digest =
            digest_header.ok_or_else(|| SignatureError::MissingHeader("Digest".to_string()))?;
        if !http_sig::verify_digest(&body_bytes, digest) {
            warn!("Digest verification failed");
            return Err(SignatureError::DigestMismatch);
        }
    }

    let request_target = format!("{} {}", method.as_str().to_lowercase(), uri.path());
    let headers_for_lookup = headers.clone();
    let ok = http_sig::verify_request(
        &public_key_pem,
        &signature,
        &request_target,
        host,
        date_header,
        |name| {
            headers_for_lookup
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        },
    )?;

    if !ok {
        warn!("HTTP signature verification failed");
        return Err(SignatureError::VerificationFailed);
    }

    info!("HTTP signature verified successfully");
    let request = Request::from_parts(parts, Body::from(body_bytes));
    Ok(next.run(request).await)
}

async fn fetch_actor_public_key(state: &AppState, key_id: &str) -> Result<String, AppError> {
    use redis::AsyncCommands;

    let cache_key = format!("actor_key:{key_id}");
    let mut redis = state.dragonfly().clone();
    let cached: Option<String> = redis.get(&cache_key).await.ok();
    if let Some(public_key_pem) = cached {
        info!("Cache hit for public key: {key_id}");
        return Ok(public_key_pem);
    }

    let actor_url = key_id.split('#').next().unwrap_or(key_id);
    info!("Fetching public key for actor: {actor_url}");

    crate::ssrf::validate_public_url(actor_url)?;

    let response = state
        .http_client()
        .get(actor_url)
        .header("Accept", "application/activity+json, application/ld+json")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch actor: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::Internal(format!(
            "Actor fetch failed with status: {}",
            response.status()
        )));
    }

    let body = crate::ssrf::read_body_limited(response, 1_048_576).await?;
    let actor_data: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::Internal(format!("Failed to parse actor JSON: {e}")))?;

    if let Some(id) = actor_data.get("id").and_then(|v| v.as_str()) {
        if id != actor_url {
            return Err(AppError::Internal(format!(
                "Actor id mismatch: expected {actor_url}, got {id}"
            )));
        }
    }

    let public_key_pem = actor_data
        .get("publicKey")
        .and_then(|pk| pk.get("publicKeyPem"))
        .and_then(|pem| pem.as_str())
        .ok_or_else(|| AppError::Internal("Public key not found in actor data".to_string()))?;

    let pem_string = public_key_pem.to_string();
    let _: Result<(), _> = redis.set_ex(&cache_key, &pem_string, 24 * 60 * 60).await;
    info!("Successfully fetched and cached public key for: {key_id}");
    Ok(pem_string)
}
