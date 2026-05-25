use axum::{extract::Request, http::header::ACCEPT_LANGUAGE, middleware::Next, response::Response};
use mithic_i18n::{DEFAULT_LOCALE, I18n};

#[derive(Debug, Clone)]
pub struct RequestLocale {
    pub locale: String,
}

impl RequestLocale {
    pub fn new(locale: String) -> Self {
        Self { locale }
    }
    pub fn as_str(&self) -> &str {
        &self.locale
    }
}

impl Default for RequestLocale {
    fn default() -> Self {
        Self {
            locale: DEFAULT_LOCALE.to_string(),
        }
    }
}

pub async fn locale_middleware(mut request: Request, next: Next) -> Response {
    let accept_language = request
        .headers()
        .get(ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok());

    let i18n = I18n::new();
    let locale = i18n.select_locale(accept_language);
    request
        .extensions_mut()
        .insert(RequestLocale::new(locale.to_string()));
    next.run(request).await
}

pub trait LocaleExt {
    fn locale(&self) -> &str;
}

impl LocaleExt for RequestLocale {
    fn locale(&self) -> &str {
        &self.locale
    }
}
