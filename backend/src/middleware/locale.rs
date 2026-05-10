use axum::{
    extract::Request,
    http::header::ACCEPT_LANGUAGE,
    middleware::Next,
    response::Response,
};

use crate::i18n::{I18n, DEFAULT_LOCALE};

/// リクエストごとのロケール情報
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

/// Accept-Languageヘッダーからロケールを抽出するミドルウェア
pub async fn locale_middleware(mut request: Request, next: Next) -> Response {
    let accept_language = request
        .headers()
        .get(ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok());

    let i18n = I18n::new();
    let locale = i18n.select_locale(accept_language);

    let request_locale = RequestLocale::new(locale.to_string());

    // Extensionにロケールを設定
    request.extensions_mut().insert(request_locale);

    // 次のミドルウェア/ハンドラへ
    next.run(request).await
}

/// ロケール情報を取得するためのヘルパーtrait
pub trait LocaleExt {
    fn locale(&self) -> &str;
}

impl LocaleExt for RequestLocale {
    fn locale(&self) -> &str {
        &self.locale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_locale_default() {
        let locale = RequestLocale::default();
        assert_eq!(locale.as_str(), "en-US");
    }

    #[test]
    fn test_request_locale_new() {
        let locale = RequestLocale::new("ja-JP".to_string());
        assert_eq!(locale.as_str(), "ja-JP");
    }
}
