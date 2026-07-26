use leptos::prelude::*;

#[component]
pub fn ErrorState(
    #[prop(default = "エラーが発生しました".into())]
    message: String,
    #[prop(default = None)]
    on_retry: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="wf-alert error" style="max-width:32rem;margin:16px auto;">
            <svg xmlns="http://www.w3.org/2000/svg" class="shrink-0" width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
            <span>{message}</span>
            {if let Some(retry) = on_retry {
                view! {
                    <button class="wf-btn wf-btn-ghost wf-btn-sm" on:click=move |_| retry.run(())>
                        "再試行"
                    </button>
                }.into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
}
