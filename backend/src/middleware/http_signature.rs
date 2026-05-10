//! HTTP Signature verification middleware for ActivityPub
//!
//! Implements HTTP Signatures as used by ActivityPub for federation.
//! Reference: https://tools.ietf.org/html/draft-cavage-http-signatures

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sigh::{Key, PublicKey};
use std::time::Duration;
use tracing::{error, info, warn};

use crate::{
    error::{AppError, Result},
    i18n::I18N,
    state::AppState,
};

/// HTTP Signature検証エラー
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

impl SignatureError {
    fn error_key(&self) -> &'static str {
        match self {
            SignatureError::MissingHeader(_) => "ap-signature-missing-header",
            SignatureError::InvalidFormat => "ap-signature-invalid-format",
            SignatureError::VerificationFailed => "ap-signature-verification-failed",
            SignatureError::DigestMismatch => "ap-signature-digest-mismatch",
            SignatureError::UnsupportedAlgorithm => "ap-signature-unsupported-algorithm",
            SignatureError::ActorKeyNotFound => "ap-signature-actor-key-not-found",
        }
    }
}

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

        // 国際化されたエラーメッセージ
        let i18n = I18N::new();
        let error_message = i18n.get_message(self.error_key(), None);

        let body = json!({
            "error": "http_signature_error",
            "message": error_message,
        });

        (status, axum::Json(body)).into_response()
    }
}

/// Parsed HTTP Signature from request headers
#[derive(Debug, Clone)]
pub struct HttpSignature {
    pub key_id: String,
    pub signature: String,
    pub headers: Vec<String>,
    pub algorithm: String,
}

impl HttpSignature {
    /// Parse Signature header
    pub fn parse(header_value: &str) -> Result<Self, SignatureError> {
        let mut key_id = None;
        let mut signature = None;
        let mut headers = vec!["(request-target)".to_string(), "host".to_string(), "date".to_string()];
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

        let key_id = key_id.ok_or(SignatureError::InvalidFormat)?;
        let signature = signature.ok_or(SignatureError::InvalidFormat)?;

        Ok(Self {
            key_id,
            signature,
            headers,
            algorithm,
        })
    }
}

/// HTTP Signature検証ミドルウェア（Inbox用）
pub async fn verify_http_signature(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, SignatureError> {
    // 必要なヘッダーを事前に収集
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| SignatureError::MissingHeader("Host".to_string()))?;

    // Signatureヘッダー確認
    let signature_header = headers
        .get("signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| SignatureError::MissingHeader("Signature".to_string()))?;

    // Dateヘッダー確認（署名検証に必要）
    let date_header = headers
        .get("date")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| SignatureError::MissingHeader("Date".to_string()))?;

    // Dateヘッダー検証（±30分ウィンドウ）- リプレイ攻撃防止
    let request_time = chrono::DateTime::parse_from_rfc2822(date_header)
        .map_err(|_| SignatureError::InvalidFormat)?;
    let now = chrono::Utc::now();
    let time_diff = (now - request_time.with_timezone(&chrono::Utc)).num_seconds().abs();

    if time_diff > 30 * 60 { // 30分を超える場合
        warn!("Request time too old or in future: {} (diff: {}s)", date_header, time_diff);
        return Err(SignatureError::VerificationFailed);
    }

    // Digestヘッダー確認（POSTリクエストの場合）
    let digest_header = headers.get("digest").and_then(|v| v.to_str().ok());

    // Signatureパース
    let signature = HttpSignature::parse(signature_header)?;

    info!("Verifying HTTP signature from key_id: {}", signature.key_id);

    // Actorの公開鍵を取得
    let public_key_pem = fetch_actor_public_key(&state, &signature.key_id)
        .await
        .map_err(|e| {
            warn!("Failed to fetch public key for {}: {}", signature.key_id, e);
            SignatureError::ActorKeyNotFound
        })?;

    // bodyを読み取る（必要な場合）
    let (parts, body) = request.into_parts();
    let body_bytes = if method == axum::http::Method::POST {
        to_bytes(body, usize::MAX)
            .await
            .map_err(|_| SignatureError::DigestMismatch)?
    } else {
        axum::body::Bytes::new()
    };

    // Digest検証（POSTの場合）
    if method == axum::http::Method::POST {
        let digest = digest_header
            .ok_or_else(|| SignatureError::MissingHeader("Digest".to_string()))?;

        if !verify_digest(&body_bytes, digest) {
            warn!("Digest verification failed");
            return Err(SignatureError::DigestMismatch);
        }
    }

    // HTTP Signature検証
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

    // リクエストを再構築して次へ
    let request = Request::from_parts(parts, Body::from(body_bytes));
    Ok(next.run(request).await)
}

/// HTTP Signature検証
fn verify_signature(
    public_key_pem: &str,
    signature: &HttpSignature,
    request_target: &str,
    host: &str,
    date: &str,
    headers: &axum::http::HeaderMap,
) -> Result<bool, SignatureError> {
    // 公開鍵をパース
    let public_key = PublicKey::from_pem(public_key_pem.as_bytes())
        .map_err(|e| {
            warn!("Failed to parse public key: {}", e);
            SignatureError::InvalidFormat
        })?;

    // 署名文字列をデコード
    let signature_bytes = base64::decode(&signature.signature)
        .map_err(|_| SignatureError::InvalidFormat)?;

    // 署名対象の文字列を構築（Signing String）
    let signing_string = build_signing_string(
        &signature.headers,
        request_target,
        host,
        date,
        headers,
    );

    // RSA-SHA256での検証
    if signature.algorithm == "rsa-sha256" || signature.algorithm == "hs2019" {
        use openssl::hash::MessageDigest;
        use openssl::pkey::PKey;
        use openssl::rsa::Rsa;
        use openssl::sign::Verifier;

        let rsa = Rsa::public_key_from_pem(public_key_pem.as_bytes())
            .map_err(|_| SignatureError::InvalidFormat)?;
        let pkey = PKey::from_rsa(rsa)
            .map_err(|_| SignatureError::InvalidFormat)?;

        let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey)
            .map_err(|_| SignatureError::VerificationFailed)?;

        verifier.update(signing_string.as_bytes())
            .map_err(|_| SignatureError::VerificationFailed)?;

        verifier.verify(&signature_bytes)
            .map_err(|_| SignatureError::VerificationFailed)
    } else {
        warn!("Unsupported algorithm: {}", signature.algorithm);
        Err(SignatureError::UnsupportedAlgorithm)
    }
}

/// Signing Stringを構築
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
            // その他のヘッダーは実装必要
            _ => continue,
        };
        parts.push(format!("{}: {}", header, value));
    }

    parts.join("\n")
}

/// Digestヘッダー検証
fn verify_digest(body: &[u8], digest_header: &str) -> bool {
    // SHA-256=base64encoded の形式を解析
    let parts: Vec<&str> = digest_header.split('=').collect();
    if parts.len() != 2 {
        return false;
    }

    let algorithm = parts[0];
    let expected_digest = parts[1];

    if !algorithm.eq_ignore_ascii_case("sha-256") && !algorithm.eq_ignore_ascii_case("SHA-256") {
        // 現時点ではSHA-256のみサポート
        return false;
    }

    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(body);
    let result = hasher.finalize();
    let actual_digest = base64::encode(&result);

    actual_digest == expected_digest
}

/// Actorの公開鍵を取得（Redisキャッシュ付き）
async fn fetch_actor_public_key(
    state: &AppState,
    key_id: &str,
) -> Result<String, AppError> {
    use redis::AsyncCommands;

    let cache_key = format!("actor_key:{}", key_id);

    // まずキャッシュを確認
    let mut redis = state.dragonfly().clone();
    let cached: Option<String> = redis.get(&cache_key).await.ok();

    if let Some(public_key_pem) = cached {
        info!("Cache hit for public key: {}", key_id);
        return Ok(public_key_pem);
    }

    // key_idは通常 "https://example.com/users/username#main-key" の形式
    // #main-key を除去してActor URLを取得
    let actor_url = key_id.split('#').next().unwrap_or(key_id);

    info!("Fetching public key for actor: {}", actor_url);

    // HTTPクライアントでActorを取得
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

    // publicKeyオブジェクトからPEMを取得
    let public_key_pem = actor_data
        .get("publicKey")
        .and_then(|pk| pk.get("publicKeyPem"))
        .and_then(|pem| pem.as_str())
        .ok_or_else(|| {
            AppError::Internal("Public key not found in actor data".to_string())
        })?;

    // キャッシュに保存（24時間TTL）
    let pem_string = public_key_pem.to_string();
    let _: Result<(), _> = redis
        .set_ex(&cache_key, &pem_string, 24 * 60 * 60)
        .await;

    info!("Successfully fetched and cached public key for: {}", key_id);
    Ok(pem_string)
}

/// Activityに署名を付与（配信用）
pub fn sign_activity(
    _actor_key: &str,
    _activity: &serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    // TODO: アクティビティ配信時の署名実装
    // 1. Actorの秘密鍵を取得
    // 2. HTTP Signatureヘッダーを生成
    // 3. Digestヘッダーを生成
    anyhow::bail!("Activity signing not implemented")
}
