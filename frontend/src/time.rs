/// RFC3339 から `HH:MM`。失敗時は `—`。
pub fn clock_hm(rfc3339: &str) -> String {
    rfc3339
        .split('T')
        .nth(1)
        .map(|t| t.chars().take(5).collect::<String>())
        .filter(|s| s.len() == 5 && s.as_bytes().get(2) == Some(&b':'))
        .unwrap_or_else(|| "—".into())
}

/// `YYYY-MM-DD` → `YYYY·MM·DD`
pub fn date_label(rfc3339: &str) -> String {
    rfc3339
        .get(..10)
        .map(|d| d.replace('-', "·"))
        .unwrap_or_default()
}

pub fn relative_label(rfc3339: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let parsed = js_sys::Date::parse(rfc3339);
        if parsed.is_nan() {
            return date_label(rfc3339);
        }
        let secs = ((js_sys::Date::now() - parsed) / 1000.0) as i64;
        if secs < 60 {
            return "いま".into();
        }
        if secs < 3600 {
            return format!("{}m", secs / 60);
        }
        if secs < 86400 {
            return format!("{}h", secs / 3600);
        }
        if secs < 86400 * 7 {
            return format!("{}d", secs / 86400);
        }
        date_label(rfc3339)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        date_label(rfc3339)
    }
}
