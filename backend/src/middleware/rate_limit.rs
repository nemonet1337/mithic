//! Fixed-window rate limit via Dragonfly (INCR + EXPIRE).
//! Works across multiple server instances sharing the same Dragonfly.

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use redis::AsyncCommands;
use std::net::SocketAddr;
use tracing::warn;

use crate::state::AppState;

#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
}

/// Auth endpoints (login / register / refresh)
pub const AUTH_RATE_LIMIT: RateLimitConfig = RateLimitConfig {
    requests_per_minute: 20,
};

fn client_key(request: &Request, trust_proxy: bool) -> String {
    if trust_proxy {
        if let Some(xff) = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
        {
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

    format!("no-connect-info:{}", request.uri().path())
}

/// Returns true if the request is allowed.
async fn check_rate_limit(
    dragonfly: &crate::db::DragonflyClient,
    key: &str,
    limit_per_minute: u32,
) -> bool {
    let redis_key = crate::db::dragonfly::rate_limit_key(key, "auth");
    let mut conn = dragonfly.clone();

    let count: u64 = match conn.incr(&redis_key, 1u64).await {
        Ok(n) => n,
        Err(e) => {
            // Fail open if cache is down (auth still has password / lockouts)
            warn!("rate limit INCR failed: {e}");
            return true;
        }
    };

    if count == 1 {
        // First hit in the window — set TTL to 60s
        let _: Result<bool, _> = conn.expire(&redis_key, 60).await;
    }

    count <= limit_per_minute as u64
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let config = AUTH_RATE_LIMIT;
    let key = client_key(&request, state.config().trust_proxy);

    if !check_rate_limit(state.dragonfly(), &key, config.requests_per_minute).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}
