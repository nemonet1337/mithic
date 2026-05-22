use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;

use crate::models::{User, sample_user};

const TOKEN_KEY: &str = "mithic.auth.token";

#[derive(Clone)]
pub struct AuthStore {
    pub token: RwSignal<Option<String>>,
    pub me: RwSignal<Option<User>>,
}

impl AuthStore {
    pub fn new() -> Self {
        let token = LocalStorage::get(TOKEN_KEY).ok();
        let me = token.as_ref().map(|_| sample_user("you", "You"));
        Self {
            token: RwSignal::new(token),
            me: RwSignal::new(me),
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
