//! Content Negotiation middleware for ActivityPub
//!
//! Handles Accept header parsing and response type selection
//! per ActivityPub specification.

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::fmt;

/// ActivityPub JSON MIME types
pub const ACTIVITY_JSON: &str = "application/activity+json";
pub const LD_JSON: &str = "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"";

/// Content type preference for response
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    ActivityJson,
    LdJson,
    Html,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::ActivityJson => write!(f, "{}", ACTIVITY_JSON),
            ContentType::LdJson => write!(f, "{}", LD_JSON),
            ContentType::Html => write!(f, "text/html"),
        }
    }
}

impl ContentType {
    /// Get the MIME type string
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::ActivityJson => ACTIVITY_JSON,
            ContentType::LdJson => LD_JSON,
            ContentType::Html => "text/html",
        }
    }

    /// Check if this is an ActivityPub content type
    pub fn is_activitypub(&self) -> bool {
        matches!(self, ContentType::ActivityJson | ContentType::LdJson)
    }
}

/// Parse Accept header and determine best content type
pub fn negotiate_content_type(accept_header: Option<&str>) -> ContentType {
    let Some(header) = accept_header else {
        return ContentType::Html;
    };

    // Parse Accept header media types with their quality values
    let mut candidates: Vec<(ContentType, f32)> = Vec::new();

    for part in header.split(',') {
        let part = part.trim();
        
        // Parse media type and q value
        let (media_type, q) = if let Some(idx) = part.find(';') {
            let (mt, params) = part.split_at(idx);
            let q = parse_q_value(params);
            (mt.trim(), q)
        } else {
            (part, 1.0)
        };

        // Match against supported types
        let content_type = match media_type {
            t if t == ACTIVITY_JSON || t == "application/activity+json; charset=utf-8" => {
                Some(ContentType::ActivityJson)
            }
            t if t.starts_with("application/ld+json") => Some(ContentType::LdJson),
            "text/html" | "application/xhtml+xml" => Some(ContentType::Html),
            "*/*" => Some(ContentType::Html), // Default for wildcard
            _ => None,
        };

        if let Some(ct) = content_type {
            candidates.push((ct, q));
        }
    }

    // Sort by quality value (highest first)
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Prefer ActivityPub types if available with reasonable quality
    for (ct, q) in candidates {
        if q > 0.0 {
            if ct.is_activitypub() {
                return ct;
            }
        }
    }

    // Default to HTML if no ActivityPub type requested
    ContentType::Html
}

/// Parse q value from parameters string
fn parse_q_value(params: &str) -> f32 {
    for param in params.split(';') {
        let param = param.trim();
        if let Some(value) = param.strip_prefix("q=") {
            return value.parse::<f32>().unwrap_or(1.0);
        }
    }
    1.0
}

/// Middleware for ActivityPub content negotiation
/// 
/// This middleware:
/// 1. Checks Accept header for ActivityPub requests
/// 2. Sets the appropriate response Content-Type
/// 3. Can reject non-ActivityPub requests for ActivityPub endpoints
pub async fn content_negotiation_middleware(
    request: Request<Body>,
    next: Next,
) -> Response {
    // Get Accept header
    let accept = request
        .headers()
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok());

    // Determine content type
    let content_type = negotiate_content_type(accept);

    // Execute the request
    let mut response = next.run(request).await;

    // Set Vary header for caching
    response.headers_mut().insert(
        header::VARY,
        header::HeaderValue::from_static("Accept"),
    );

    // Set Content-Type based on negotiation
    if content_type.is_activitypub() {
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static(content_type.as_str()),
        );
    }

    response
}

/// Check if request is an ActivityPub request
pub fn is_activitypub_request(accept_header: Option<&str>) -> bool {
    negotiate_content_type(accept_header).is_activitypub()
}

/// Extension trait for Request to add content negotiation helpers
pub trait ContentNegotiationExt {
    fn content_type(&self) -> ContentType;
    fn is_activitypub(&self) -> bool;
}

impl ContentNegotiationExt for Request<Body> {
    fn content_type(&self) -> ContentType {
        let accept = self
            .headers()
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok());
        negotiate_content_type(accept)
    }

    fn is_activitypub(&self) -> bool {
        self.content_type().is_activitypub()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negotiate_activity_json() {
        let result = negotiate_content_type(Some("application/activity+json"));
        assert_eq!(result, ContentType::ActivityJson);
    }

    #[test]
    fn test_negotiate_ld_json() {
        let result = negotiate_content_type(Some("application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\""));
        assert_eq!(result, ContentType::LdJson);
    }

    #[test]
    fn test_negotiate_html() {
        let result = negotiate_content_type(Some("text/html"));
        assert_eq!(result, ContentType::Html);
    }

    #[test]
    fn test_negotiate_prefer_activitypub() {
        let result = negotiate_content_type(Some("text/html, application/activity+json"));
        assert!(result.is_activitypub());
    }

    #[test]
    fn test_negotiate_with_quality() {
        let result = negotiate_content_type(Some("text/html;q=0.9, application/activity+json;q=0.8"));
        assert_eq!(result, ContentType::Html);
    }

    #[test]
    fn test_negotiate_activitypub_wins_with_quality() {
        let result = negotiate_content_type(Some("text/html;q=0.8, application/activity+json;q=0.9"));
        assert_eq!(result, ContentType::ActivityJson);
    }
}
