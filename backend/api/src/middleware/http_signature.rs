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
use tracing::{info, warn};

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

    // POST では request-target / host / date / digest が署名対象に含まれることを必須化
    if method == axum::http::Method::POST {
        let required = ["(request-target)", "host", "date", "digest"];
        for req in required {
            if !signature
                .headers
                .iter()
                .any(|h| h.eq_ignore_ascii_case(req))
            {
                warn!("Signature missing required header binding: {}", req);
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

/// Reconstruct the HTTP signature signing string from the headers listed in the
/// `Signature` header, in order. Returns `None` if a referenced header is absent.
fn build_signing_string(
    signature: &HttpSignature,
    request_target: &str,
    host: &str,
    date: &str,
    headers: &axum::http::HeaderMap,
) -> Option<String> {
    let mut lines = Vec::with_capacity(signature.headers.len());
    for name in &signature.headers {
        let value = match name.as_str() {
            "(request-target)" => request_target.to_string(),
            "host" => host.to_string(),
            "date" => date.to_string(),
            other => headers
                .get(other)
                .and_then(|v| v.to_str().ok())?
                .to_string(),
        };
        lines.push(format!("{}: {}", name, value));
    }
    Some(lines.join("\n"))
}

fn verify_signature(
    public_key_pem: &str,
    signature: &HttpSignature,
    request_target: &str,
    host: &str,
    date: &str,
    headers: &axum::http::HeaderMap,
) -> Result<bool, SignatureError> {
    use rsa::pkcs1v15::VerifyingKey;
    use rsa::sha2::Sha256;
    use rsa::signature::Verifier;
    use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey};

    if signature.algorithm != "rsa-sha256" && signature.algorithm != "hs2019" {
        warn!("Unsupported algorithm: {}", signature.algorithm);
        return Err(SignatureError::UnsupportedAlgorithm);
    }

    let Some(signing_string) = build_signing_string(signature, request_target, host, date, headers)
    else {
        warn!("A signed header referenced in the Signature is missing from the request");
        return Ok(false);
    };

    let signature_bytes = match BASE64_STANDARD.decode(signature.signature.as_bytes()) {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!("Failed to base64-decode signature: {}", e);
            return Ok(false);
        }
    };

    // Parse the public key. Try PKCS#8 first, then fall back to PKCS#1.
    let pkey = match RsaPublicKey::from_public_key_pem(public_key_pem) {
        Ok(key) => key,
        Err(_) => match RsaPublicKey::from_pkcs1_pem(public_key_pem) {
            Ok(key) => key,
            Err(e) => {
                warn!("Failed to parse public key PEM as PKCS#8 or PKCS#1: {}", e);
                return Ok(false);
            }
        },
    };

    let verifying_key = VerifyingKey::<Sha256>::new(pkey);
    let signature = match rsa::pkcs1v15::Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(e) => {
            warn!("Failed to parse signature: {}", e);
            return Ok(false);
        }
    };
    match verifying_key.verify(signing_string.as_bytes(), &signature) {
        Ok(_) => Ok(true),
        Err(e) => {
            warn!("Signature verification failed: {}", e);
            Ok(false)
        }
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn verify_digest(body: &[u8], digest_header: &str) -> bool {
    let parts: Vec<&str> = digest_header.splitn(2, '=').collect();
    if parts.len() != 2 {
        return false;
    }
    if !parts[0].eq_ignore_ascii_case("sha-256") {
        return false;
    }

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(body);
    let actual_digest = BASE64_STANDARD.encode(hasher.finalize());
    constant_time_eq(actual_digest.as_bytes(), parts[1].as_bytes())
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

    crate::ssrf::validate_public_url(actor_url)?;

    let response = state
        .http_client()
        .get(actor_url)
        .header("Accept", "application/activity+json, application/ld+json")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch actor: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Internal(format!(
            "Actor fetch failed with status: {}",
            response.status()
        )));
    }

    let body = crate::ssrf::read_body_limited(response, 1_048_576).await?;
    let actor_data: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::Internal(format!("Failed to parse actor JSON: {}", e)))?;

    // 取得元 URL と JSON id の一致を検証 (なりすまし防止)
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
    info!("Successfully fetched and cached public key for: {}", key_id);
    Ok(pem_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use rsa::pkcs1v15::SigningKey;
    use rsa::sha2::Sha256;
    use rsa::signature::{RandomizedSigner, SignatureEncoding};
    use rsa::{RsaPrivateKey, RsaPublicKey, pkcs8::EncodePublicKey};

    /// Generate an RSA-2048 key pair, returning (RsaPrivateKey, public key PEM string).
    fn gen_keypair() -> (RsaPrivateKey, String) {
        let mut rng = thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        let public_pem = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();
        (private_key, public_pem)
    }

    /// Sign `data` with `pkey` using RSA-SHA256 and return the base64-encoded signature.
    fn sign_b64(pkey: &RsaPrivateKey, data: &str) -> String {
        let mut rng = thread_rng();
        let signing_key = SigningKey::<Sha256>::new(pkey.clone());
        let signature = signing_key.sign_with_rng(&mut rng, data.as_bytes());
        BASE64_STANDARD.encode(signature.to_bytes())
    }

    fn make_signature(headers: &[&str], sig_b64: &str, algorithm: &str) -> HttpSignature {
        HttpSignature {
            key_id: "https://example.com/users/alice#main-key".to_string(),
            signature: sig_b64.to_string(),
            headers: headers.iter().map(|s| s.to_string()).collect(),
            algorithm: algorithm.to_string(),
        }
    }

    #[test]
    fn build_signing_string_orders_and_formats_headers() {
        let mut header_map = axum::http::HeaderMap::new();
        header_map.insert("digest", "SHA-256=abc".parse().unwrap());
        let sig = make_signature(
            &["(request-target)", "host", "date", "digest"],
            "",
            "rsa-sha256",
        );

        let result = build_signing_string(
            &sig,
            "post /inbox",
            "example.com",
            "Tue, 20 Apr 2021 02:07:55 GMT",
            &header_map,
        )
        .unwrap();

        assert_eq!(
            result,
            "(request-target): post /inbox\n\
             host: example.com\n\
             date: Tue, 20 Apr 2021 02:07:55 GMT\n\
             digest: SHA-256=abc"
        );
    }

    #[test]
    fn build_signing_string_returns_none_for_missing_header() {
        let header_map = axum::http::HeaderMap::new();
        let sig = make_signature(&["(request-target)", "digest"], "", "rsa-sha256");
        let result =
            build_signing_string(&sig, "post /inbox", "example.com", "some-date", &header_map);
        assert!(result.is_none());
    }

    #[test]
    fn verify_signature_accepts_valid_signature() {
        let (pkey, public_pem) = gen_keypair();
        let request_target = "post /inbox";
        let host = "example.com";
        let date = "Tue, 20 Apr 2021 02:07:55 GMT";

        let signing_string =
            format!("(request-target): {request_target}\nhost: {host}\ndate: {date}");
        let sig_b64 = sign_b64(&pkey, &signing_string);
        let sig = make_signature(
            &["(request-target)", "host", "date"],
            &sig_b64,
            "rsa-sha256",
        );

        let headers = axum::http::HeaderMap::new();
        let result =
            verify_signature(&public_pem, &sig, request_target, host, date, &headers).unwrap();
        assert!(result, "valid signature should verify");
    }

    #[test]
    fn verify_signature_rejects_tampered_signature() {
        let (pkey, public_pem) = gen_keypair();
        let request_target = "post /inbox";
        let host = "example.com";
        let date = "Tue, 20 Apr 2021 02:07:55 GMT";

        let signing_string =
            format!("(request-target): {request_target}\nhost: {host}\ndate: {date}");
        // Sign the correct string, but verify against a different request target.
        let sig_b64 = sign_b64(&pkey, &signing_string);
        let sig = make_signature(
            &["(request-target)", "host", "date"],
            &sig_b64,
            "rsa-sha256",
        );

        let headers = axum::http::HeaderMap::new();
        let result =
            verify_signature(&public_pem, &sig, "post /other", host, date, &headers).unwrap();
        assert!(!result, "signature over different data must be rejected");
    }

    #[test]
    fn verify_signature_rejects_wrong_key() {
        let (pkey, _public_pem) = gen_keypair();
        let (_other_pkey, other_public_pem) = gen_keypair();
        let request_target = "post /inbox";
        let host = "example.com";
        let date = "Tue, 20 Apr 2021 02:07:55 GMT";

        let signing_string =
            format!("(request-target): {request_target}\nhost: {host}\ndate: {date}");
        let sig_b64 = sign_b64(&pkey, &signing_string);
        let sig = make_signature(
            &["(request-target)", "host", "date"],
            &sig_b64,
            "rsa-sha256",
        );

        let headers = axum::http::HeaderMap::new();
        let result = verify_signature(
            &other_public_pem,
            &sig,
            request_target,
            host,
            date,
            &headers,
        )
        .unwrap();
        assert!(
            !result,
            "signature must not verify against an unrelated key"
        );
    }

    #[test]
    fn verify_signature_rejects_unsupported_algorithm() {
        let (_pkey, public_pem) = gen_keypair();
        let sig = make_signature(&["(request-target)", "host", "date"], "AAAA", "ed25519");
        let headers = axum::http::HeaderMap::new();
        let result = verify_signature(
            &public_pem,
            &sig,
            "post /inbox",
            "example.com",
            "some-date",
            &headers,
        );
        assert!(matches!(result, Err(SignatureError::UnsupportedAlgorithm)));
    }

    #[test]
    fn verify_signature_rejects_invalid_base64() {
        let (_pkey, public_pem) = gen_keypair();
        let sig = make_signature(
            &["(request-target)", "host", "date"],
            "not valid base64!!!",
            "rsa-sha256",
        );
        let headers = axum::http::HeaderMap::new();
        let result = verify_signature(
            &public_pem,
            &sig,
            "post /inbox",
            "example.com",
            "some-date",
            &headers,
        )
        .unwrap();
        assert!(!result, "non-base64 signature must be rejected, not error");
    }
}
