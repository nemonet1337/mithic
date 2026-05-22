use axum::{extract::State, http::StatusCode, middleware::Next, response::Response};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<std::collections::HashMap<String, TokenBucket>>>,
}

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: u32,
    max_tokens: u32,
    refill_rate: u32,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: u32, refill_rate: u32) -> Self {
        Self { tokens: max_tokens, max_tokens, refill_rate, last_refill: Instant::now() }
    }

    fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        if self.tokens >= tokens { self.tokens -= tokens; true } else { false }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = (elapsed.as_secs() as u32) * self.refill_rate;
        if tokens_to_add > 0 {
            self.tokens = (self.tokens + tokens_to_add).min(self.max_tokens);
            self.last_refill = now;
        }
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self { buckets: Arc::new(Mutex::new(std::collections::HashMap::new())) }
    }

    pub async fn check_rate_limit(&self, key: &str, limit: u32) -> bool {
        let mut buckets = self.buckets.lock().await;
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(limit, limit / 60));
        bucket.try_consume(1)
    }

    pub async fn cleanup(&self) {
        let mut buckets = self.buckets.lock().await;
        buckets.retain(|_, bucket| bucket.last_refill.elapsed() < Duration::from_secs(3600));
    }
}

impl Default for RateLimiter {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self { Self { requests_per_minute: 60, burst_size: 10 } }
}

pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let config = RateLimitConfig::default();
    let client_id = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| request.headers().get("x-real-ip").and_then(|v| v.to_str().ok()))
        .unwrap_or("unknown")
        .to_string();

    if !limiter.check_rate_limit(&client_id, config.requests_per_minute).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}
