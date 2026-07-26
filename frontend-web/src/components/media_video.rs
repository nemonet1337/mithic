use leptos::prelude::*;

#[component]
pub fn MediaVideo(
    url: String,
    #[prop(default = None)]
    preview_url: Option<String>,
) -> impl IntoView {
    let playing = RwSignal::new(false);

    view! {
        <div class="wf-thumb relative overflow-hidden">
            <Show
                when=move || playing.get()
                fallback=move || {
                    view! {
                        <div class="relative aspect-video cursor-pointer" style="background:var(--paper-3);"
                             on:click=move |_| playing.set(true)>
                            {if let Some(ref thumb) = preview_url {
                                view! {
                                    <img src=thumb.clone() class="w-full h-full object-cover" loading="lazy" alt="video preview" />
                                }.into_any()
                            } else {
                                view! {
                                    <div class="flex items-center justify-center w-full h-full" />
                                }.into_any()
                            }}
                            <div class="absolute inset-0 flex items-center justify-center bg-black/30">
                                <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="white" stroke="white" stroke-width="1"><polygon points="5 3 19 12 5 21 5 3"/></svg>
                            </div>
                        </div>
                    }
                }
            >
                <video
                    src=url.clone()
                    class="w-full max-h-96"
                    controls
                    autoplay
                />
            </Show>
        </div>
    }
}
