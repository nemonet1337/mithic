use leptos::prelude::*;

#[component]
pub fn ErrorState(
    #[prop(default = "エラーが発生しました".into())]
    message: String,
    #[prop(default = None)]
    on_retry: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="alert alert-error shadow-lg max-w-lg mx-auto my-4">
            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
            <span>{message}</span>
            {if let Some(retry) = on_retry {
                view! {
                    <button class="btn btn-sm btn-ghost" on:click=move |_| retry.call(())>
                        "再試行"
                    </button>
                }.into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
}
