use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::store::AuthStore;

#[component]
pub fn Protected(children: Children) -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let auth_for_redirect = auth.clone();
    let navigate = use_navigate();
    Effect::new(move |_| {
        if !auth_for_redirect.is_authenticated() {
            navigate("/login", Default::default());
        }
    });
    if auth.is_authenticated() {
        children().into_any()
    } else {
        ().into_any()
    }
}
