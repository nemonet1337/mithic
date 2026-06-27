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
    let btn_class = match size {
        "xs" => "btn-xs",
        "lg" => "btn-lg",
        _ => "btn-sm",
    };

    view! {
        <button
            class=move || {
                if is_pending {
                    format!("btn {} btn-disabled", btn_class)
                } else if is_following {
                    format!("btn {} btn-outline", btn_class)
                } else {
                    format!("btn {} btn-primary", btn_class)
                }
            }
            disabled=move || is_pending
            on:click=move |_| on_toggle.call(())
        >
            {if is_pending {
                view! { <span class="loading loading-spinner loading-xs" /> }.into_any()
            } else if is_following {
                view! { "フォロー中" }.into_any()
            } else {
                view! { "フォロー" }.into_any()
            }}
        </button>
    }
}
