use leptos::prelude::*;

#[component]
pub fn MediaImage(
    url: String,
    #[prop(default = None)]
    alt: Option<String>,
    #[prop(default = None)]
    preview_url: Option<String>,
    #[prop(default = false)]
    is_sensitive: bool,
) -> impl IntoView {
    let revealed = RwSignal::new(!is_sensitive);

    view! {
        <div class="relative overflow-hidden rounded-xl border border-base-300">
            <Show
                when=move || revealed.get()
                fallback=move || {
                    view! {
                        <div class="flex items-center justify-center bg-base-200 aspect-video cursor-pointer"
                             on:click=move |_| revealed.set(true)>
                            <div class="text-center">
                                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="mx-auto mb-1 opacity-50"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 9.9-1"/></svg>
                                <span class="text-xs opacity-60">"センシティブ"</span>
                            </div>
                        </div>
                    }
                }
            >
                <img
                    src=if let Some(ref thumb) = preview_url { thumb.clone() } else { url.clone() }
                    alt=alt.clone().unwrap_or_default()
                    class="w-full h-auto max-h-96 object-cover cursor-pointer"
                    loading="lazy"
                    on:click=move |_| {
                        let window = web_sys::window().unwrap();
                        let _ = window.open_with_url_and_target(&url, "_blank");
                    }
                />
            </Show>
        </div>
    }
}
