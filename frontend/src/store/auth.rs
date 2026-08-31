use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;

use shared::User;

const TOKEN_KEY: &str = "mithic.auth.token";

#[derive(Clone)]
pub struct AuthStore {
    pub token: RwSignal<Option<String>>,
    pub me: RwSignal<Option<User>>,
    /// `/users/me` 検証が終わるまで false。未検証トークンで保護画面を開かない。
    pub ready: RwSignal<bool>,
}

impl AuthStore {
    pub fn new() -> Self {
        let token: Option<String> = LocalStorage::get(TOKEN_KEY).ok();
        let ready = token.is_none();
        Self {
            token: RwSignal::new(token),
            me: RwSignal::new(None),
            ready: RwSignal::new(ready),
        }
    }

    /// 起動時にトークンを /me で検証し、失敗したらログアウトする。
    pub fn verify_on_startup(&self) {
        let store = self.clone();
        let Some(token) = self.token.get_untracked() else {
            store.ready.set(true);
            return;
        };
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::auth::fetch_me(&token).await {
                Ok(user) => {
                    if store.token.get_untracked().as_deref() == Some(token.as_str()) {
                        store.me.set(Some(user));
                    }
                    store.ready.set(true);
                }
                Err(_) => {
                    if store.token.get_untracked().as_deref() == Some(token.as_str()) {
                        store.logout();
                    }
                    store.ready.set(true);
                }
            }
        });
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.get().is_some() && self.me.get().is_some()
    }

    pub fn login(&self, token: String, user: User) {
        if let Err(error) = LocalStorage::set(TOKEN_KEY, &token) {
            web_sys::console::warn_1(&format!("failed to persist auth token: {error:?}").into());
        }
        self.token.set(Some(token));
        self.me.set(Some(user));
        self.ready.set(true);
    }

    pub fn logout(&self) {
        LocalStorage::delete(TOKEN_KEY);
        self.token.set(None);
        self.me.set(None);
        self.ready.set(true);
    }
}
