use std::collections::HashMap;

use fluent::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::langid;

/// サポートされるロケール
pub const SUPPORTED_LOCALES: &[&str] = &["en-US", "ja-JP"];

/// デフォルトロケール
pub const DEFAULT_LOCALE: &str = "en-US";

/// 国際化システム
pub struct I18n {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
}

impl I18n {
    /// 新しいI18nインスタンスを作成
    pub fn new() -> Self {
        let mut bundles = HashMap::new();

        // 英語リソース
        let en_resource = FluentResource::try_new(include_str!("../locales/en.ftl").to_string())
            .expect("Failed to parse English FTL");
        let en_lang = langid!("en-US");
        let mut en_bundle = FluentBundle::new(vec![en_lang]);
        en_bundle
            .add_resource(en_resource)
            .expect("Failed to add English resource");
        bundles.insert("en-US".to_string(), en_bundle);

        // 日本語リソース
        let ja_resource = FluentResource::try_new(include_str!("../locales/ja.ftl").to_string())
            .expect("Failed to parse Japanese FTL");
        let ja_lang = langid!("ja-JP");
        let mut ja_bundle = FluentBundle::new(vec![ja_lang]);
        ja_bundle
            .add_resource(ja_resource)
            .expect("Failed to add Japanese resource");
        bundles.insert("ja-JP".to_string(), ja_bundle);

        Self { bundles }
    }

    /// メッセージを翻訳
    pub fn translate(&self, locale: &str, key: &str) -> String {
        self.translate_with_args(locale, key, None)
    }

    /// 引数付きで翻訳
    pub fn translate_with_args(
        &self,
        locale: &str,
        key: &str,
        args: Option<&FluentArgs>,
    ) -> String {
        let bundle = self.bundles.get(locale).unwrap_or_else(|| {
            self.bundles
                .get(DEFAULT_LOCALE)
                .expect("Default locale must exist")
        });

        let message = match bundle.get_message(key) {
            Some(m) => m,
            None => return format!("[missing: {}]", key),
        };

        let pattern = match message.value() {
            Some(p) => p,
            None => return format!("[no-value: {}]", key),
        };

        let mut errors = vec![];
        let value = bundle.format_pattern(pattern, args, &mut errors);

        if !errors.is_empty() {
            tracing::warn!("Translation errors for key '{}': {:?}", key, errors);
        }

        value.to_string()
    }

    /// Accept-Languageヘッダーから最適なロケールを選択
    pub fn select_locale(&self, accept_language: Option<&str>) -> &str {
        let Some(header) = accept_language else {
            return DEFAULT_LOCALE;
        };

        for part in header.split(',') {
            let lang = part.split(';').next().unwrap_or(part).trim();
            if self.bundles.contains_key(lang) {
                return SUPPORTED_LOCALES
                    .iter()
                    .find(|&&l| l == lang)
                    .copied()
                    .unwrap_or(DEFAULT_LOCALE);
            }
            // 部分一致チェック (例: "en" -> "en-US")
            for &supported in SUPPORTED_LOCALES {
                if supported.starts_with(lang)
                    || lang.starts_with(supported.split('-').next().unwrap_or(supported))
                {
                    return supported;
                }
            }
        }

        DEFAULT_LOCALE
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}


/// 簡易翻訳マクロ
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::I18N.translate($crate::DEFAULT_LOCALE, $key)
    };
    ($key:expr, $locale:expr) => {
        $crate::I18N.translate($locale, $key)
    };
}

#[macro_export]
macro_rules! t_args {
    ($key:expr, $($name:ident = $value:expr),*) => {{
        let mut args = fluent::FluentArgs::new();
        $(args.set(stringify!($name), $value);)*
        $crate::I18N.translate_with_args($crate::DEFAULT_LOCALE, $key, Some(&args))
    }};
    ($key:expr, $locale:expr, $($name:ident = $value:expr),*) => {{
        let mut args = fluent::FluentArgs::new();
        $(args.set(stringify!($name), $value);)*
        $crate::I18N.translate_with_args($locale, $key, Some(&args))
    }};
}
