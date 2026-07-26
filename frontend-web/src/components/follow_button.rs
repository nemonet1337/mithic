use leptos::prelude::*;

#[component]
pub fn FollowButton(
    is_following: bool,
    #[prop(default = false)]
    is_pending: bool,
    on_toggle: Callback<()>,
    #[prop(default = "sm")]
    size: &'static str,
) -> impl IntoView {
    let _ = size;

    view! {
        <button
            class=move || {
                if is_following { "wf-follow-pill following" } else { "wf-follow-pill" }
            }
            disabled=move || is_pending
            on:click=move |_| on_toggle.run(())
        >
            {if is_pending {
                view! { <span class="wf-spinner" style="width:12px;height:12px;border-width:2px;" /> }.into_any()
            } else if is_following {
                view! { "フォロー中" }.into_any()
            } else {
                view! { "フォロー" }.into_any()
            }}
        </button>
    }
}
