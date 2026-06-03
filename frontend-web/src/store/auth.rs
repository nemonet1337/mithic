use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;

use crate::models::User;

const TOKEN_KEY: &str = "mithic.auth.token";

#[derive(Clone)]
pub struct AuthStore {
    pub token: RwSignal<Option<String>>,
    pub me: RwSignal<Option<User>>,
}

impl AuthStore {
    pub fn new() -> Self {
        let token = LocalStorage::get(TOKEN_KEY).ok();
        Self {
            token: RwSignal::new(token),
            me: RwSignal::new(None),
        }
    }

    /// 起動時にトークンを /me で検証し、失敗したらログアウトする。
    pub fn verify_on_startup(&self) {
        let store = self.clone();
        if let Some(token) = self.token.get_untracked() {
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::auth::fetch_me(&token).await {
                    Ok(user) => store.me.set(Some(user)),
                    Err(_) => store.logout(),
                }
            });
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.get().is_some()
    }

    pub fn login(&self, token: String, user: User) {
        if let Err(error) = LocalStorage::set(TOKEN_KEY, &token) {
            web_sys::console::warn_1(&format!("failed to persist auth token: {error:?}").into());
        }
        self.token.set(Some(token));
        self.me.set(Some(user));
    }

    pub fn logout(&self) {
        LocalStorage::delete(TOKEN_KEY);
        self.token.set(None);
        self.me.set(None);
    }
}
