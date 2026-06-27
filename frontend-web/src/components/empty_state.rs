use leptos::prelude::*;

#[component]
pub fn EmptyState(
    #[prop(default = "データがありません".into())]
    message: String,
    #[prop(default = "".into())]
    icon: String,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center gap-2 py-16 text-base-content/40">
            {if !icon.is_empty() {
                view! { <span class="text-4xl">{icon}</span> }.into_any()
            } else {
                view! {
                    <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>
                }.into_any()
            }}
            <span class="text-sm">{message}</span>
        </div>
    }
}
