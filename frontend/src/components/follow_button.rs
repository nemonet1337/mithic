use leptos::prelude::*;

#[component]
pub fn FollowButton(
    #[prop(into)] is_following: Signal<bool>,
    #[prop(into)] is_pending: Signal<bool>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                if is_following.get() {
                    "wf-follow-pill following"
                } else {
                    "wf-follow-pill"
                }
            }
            disabled=move || is_pending.get()
            on:click=move |_| on_toggle.run(())
        >
            {move || {
                if is_pending.get() {
                    view! { <span class="wf-spinner" style="width:12px;height:12px;border-width:2px;" /> }.into_any()
                } else if is_following.get() {
                    view! { "フォロー中" }.into_any()
                } else {
                    view! { "フォロー" }.into_any()
                }
            }}
        </button>
    }
}
