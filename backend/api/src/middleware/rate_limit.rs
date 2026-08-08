use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::state::AppState;

/// プロセス内トークンバケツ。単一インスタンス前提。
/// 水平スケール時は Dragonfly INCR+EXPIRE へ置換すること。
#[derive(Debug, Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<std::collections::HashMap<String, TokenBucket>>>,
}

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    /// tokens per second
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: u32, refill_per_minute: u32) -> Self {
        let max = max_tokens.max(1) as f64;
        let per_sec = (refill_per_minute.max(1) as f64) / 60.0;
        Self {
            tokens: max,
            max_tokens: max,
            refill_rate: per_sec,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
            self.last_refill = now;
        }
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn check_rate_limit(&self, key: &str, limit_per_minute: u32, burst: u32) -> bool {
        let mut buckets = self.buckets.lock().await;
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(burst.max(limit_per_minute), limit_per_minute));
        bucket.try_consume(1.0)
    }

    pub async fn cleanup(&self) {
        let mut buckets = self.buckets.lock().await;
        buckets.retain(|_, bucket| bucket.last_refill.elapsed() < Duration::from_secs(3600));
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
        }
    }
}

/// 認証系向けの厳しい制限
pub const AUTH_RATE_LIMIT: RateLimitConfig = RateLimitConfig {
    requests_per_minute: 20,
    burst_size: 5,
};

fn client_key(request: &Request, trust_proxy: bool) -> String {
    if trust_proxy {
        if let Some(xff) = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
        {
            // 左端 (クライアント) を採用
            if let Some(first) = xff.split(',').next() {
                let ip = first.trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
        if let Some(real_ip) = request
            .headers()
            .get("x-real-ip")
            .and_then(|v| v.to_str().ok())
        {
            let ip = real_ip.trim();
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }

    if let Some(ConnectInfo(addr)) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        return addr.ip().to_string();
    }

    // ConnectInfo が無い場合は接続ごとに区別できないため "unknown" は使わず
    // 全未知を共有バケツにしないよう path 付きキーにする
    format!("no-connect-info:{}", request.uri().path())
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let config = AUTH_RATE_LIMIT;
    let key = client_key(&request, state.config().trust_proxy);

    if !state
        .rate_limiter()
        .check_rate_limit(&key, config.requests_per_minute, config.burst_size)
        .await
    {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}
