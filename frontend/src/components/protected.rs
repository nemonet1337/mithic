use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::store::AuthStore;

#[component]
pub fn Protected(children: ChildrenFn) -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let auth_effect = auth.clone();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if !auth_effect.is_authenticated() {
            navigate("/login", Default::default());
        }
    });

    view! {
        <Show
            when=move || auth.is_authenticated()
            fallback=|| {
                view! {
                    <div class="flex items-center justify-center min-h-screen gap-2">
                        <span class="wf-spinner" style="width:20px;height:20px;" />
                        <span class="wf-entry-meta">"読み込み中…"</span>
                    </div>
                }
            }
        >
            {children()}
        </Show>
    }
}
