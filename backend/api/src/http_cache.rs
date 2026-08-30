//! 公開 GET 向け ETag / Cache-Control ヘルパ

use axum::{
    Json,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sha2::{Digest, Sha256};

/// ボディから弱 ETag を生成する
pub fn weak_etag<T: Serialize>(body: &T) -> String {
    let bytes = serde_json::to_vec(body).unwrap_or_default();
    let hash = Sha256::digest(&bytes);
    // 先頭 16 hex = 8 bytes で十分 (衝突耐性より帯域)
    format!("W/\"{}\"", hex::encode(&hash[..8]))
}

/// If-None-Match が ETag と一致するか
pub fn etag_matches(headers: &HeaderMap, etag: &str) -> bool {
    let Some(raw) = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    raw.split(',')
        .map(str::trim)
        .any(|candidate| candidate == "*" || candidate == etag)
}

/// JSON + Cache-Control + ETag。条件付きで 304。
pub fn json_with_cache<T: Serialize>(
    headers: &HeaderMap,
    body: T,
    cache_control: &str,
) -> Response {
    let etag = weak_etag(&body);
    if etag_matches(headers, &etag) {
        let mut res = StatusCode::NOT_MODIFIED.into_response();
        if let Ok(v) = HeaderValue::from_str(&etag) {
            res.headers_mut().insert(header::ETAG, v);
        }
        if let Ok(v) = HeaderValue::from_str(cache_control) {
            res.headers_mut().insert(header::CACHE_CONTROL, v);
        }
        return res;
    }

    let mut res = Json(body).into_response();
    if let Ok(v) = HeaderValue::from_str(&etag) {
        res.headers_mut().insert(header::ETAG, v);
    }
    if let Ok(v) = HeaderValue::from_str(cache_control) {
        res.headers_mut().insert(header::CACHE_CONTROL, v);
    }
    res
}

/// タイムライン先頭ページ: 短命キャッシュ
pub const CC_TIMELINE: &str = "public, max-age=15, stale-while-revalidate=30";
/// トレンドハッシュタグ
pub const CC_TRENDING: &str = "public, max-age=60, stale-while-revalidate=120";
/// インスタンスメタ
pub const CC_INSTANCE: &str = "public, max-age=120, stale-while-revalidate=300";
/// 公開ノート単体
pub const CC_PUBLIC_NOTE: &str = "public, max-age=30, must-revalidate";
