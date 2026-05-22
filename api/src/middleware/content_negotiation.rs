use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::fmt;

pub const ACTIVITY_JSON: &str = "application/activity+json";
pub const LD_JSON: &str = "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType { ActivityJson, LdJson, Html }

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
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::ActivityJson => ACTIVITY_JSON,
            ContentType::LdJson => LD_JSON,
            ContentType::Html => "text/html",
        }
    }

    pub fn is_activitypub(&self) -> bool {
        matches!(self, ContentType::ActivityJson | ContentType::LdJson)
    }
}

pub fn negotiate_content_type(accept_header: Option<&str>) -> ContentType {
    let Some(header) = accept_header else { return ContentType::Html; };
    let mut candidates: Vec<(ContentType, f32)> = Vec::new();

    for part in header.split(',') {
        let part = part.trim();
        let (media_type, q) = if let Some(idx) = part.find(';') {
            let (mt, params) = part.split_at(idx);
            (mt.trim(), parse_q_value(params))
        } else {
            (part, 1.0)
        };

        let content_type = match media_type {
            t if t == ACTIVITY_JSON || t == "application/activity+json; charset=utf-8" => Some(ContentType::ActivityJson),
            t if t.starts_with("application/ld+json") => Some(ContentType::LdJson),
            "text/html" | "application/xhtml+xml" => Some(ContentType::Html),
            "*/*" => Some(ContentType::Html),
            _ => None,
        };

        if let Some(ct) = content_type { candidates.push((ct, q)); }
    }

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (ct, q) in candidates {
        if q > 0.0 && ct.is_activitypub() { return ct; }
    }
    ContentType::Html
}

fn parse_q_value(params: &str) -> f32 {
    for param in params.split(';') {
        let param = param.trim();
        if let Some(value) = param.strip_prefix("q=") {
            return value.parse::<f32>().unwrap_or(1.0);
        }
    }
    1.0
}

pub async fn content_negotiation_middleware(request: Request<Body>, next: Next) -> Response {
    let accept = request.headers().get(header::ACCEPT).and_then(|v| v.to_str().ok());
    let content_type = negotiate_content_type(accept);
    let mut response = next.run(request).await;
    response.headers_mut().insert(header::VARY, header::HeaderValue::from_static("Accept"));
    if content_type.is_activitypub() {
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static(content_type.as_str()),
        );
    }
    response
}

pub fn is_activitypub_request(accept_header: Option<&str>) -> bool {
    negotiate_content_type(accept_header).is_activitypub()
}

pub trait ContentNegotiationExt {
    fn content_type(&self) -> ContentType;
    fn is_activitypub(&self) -> bool;
}

impl ContentNegotiationExt for Request<Body> {
    fn content_type(&self) -> ContentType {
        let accept = self.headers().get(header::ACCEPT).and_then(|v| v.to_str().ok());
        negotiate_content_type(accept)
    }

    fn is_activitypub(&self) -> bool { self.content_type().is_activitypub() }
}
