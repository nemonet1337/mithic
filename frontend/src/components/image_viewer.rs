use leptos::ev;
use leptos::leptos_dom::helpers::window_event_listener;
use leptos::prelude::*;
use leptos_icons::Icon;

use icondata as id;
use shared::MediaAttachment;

/// 添付画像の原解像度ビューア。開閉と現在位置は呼び出し側（`MediaThumbs`）が持つ。
#[component]
pub fn ImageViewer(
    images: Vec<MediaAttachment>,
    index: RwSignal<usize>,
    on_close: Callback<()>,
) -> impl IntoView {
    let len = images.len();
    let multiple = len > 1;
    let step = move |delta: isize| {
        if len == 0 {
            return;
        }
        let next = (index.get_untracked() as isize + delta).rem_euclid(len as isize);
        index.set(next as usize);
    };
    let keys = window_event_listener(ev::keydown, move |e: web_sys::KeyboardEvent| {
        match e.key().as_str() {
            "ArrowLeft" => step(-1),
            "ArrowRight" => step(1),
            "Escape" => on_close.run(()),
            _ => {}
        }
    });
    on_cleanup(move || keys.remove());

    let current = Signal::derive(move || images.get(index.get_untracked()).cloned());
    let url = Signal::derive(move || current.get().map(|a| a.url).unwrap_or_default());
    let alt = Signal::derive(move || current.get().and_then(|a| a.alt).unwrap_or_default());

    view! {
        <div class="wf-overlay" on:click=move |_| on_close.run(())>
            <div
                class="relative flex w-full max-w-[94vw] flex-col items-center gap-2"
                on:click=move |ev| ev.stop_propagation()
            >
                <img
                    src=move || url.get()
                    alt=move || alt.get()
                    class="max-h-[86dvh] max-w-full rounded-wf object-contain shadow-hard"
                />
                <Show when=move || multiple>
                    <button
                        class="absolute top-1/2 left-1 -translate-y-1/2 rounded-full border-0 bg-black/55 p-2 text-white cursor-pointer"
                        aria-label="前の画像"
                        on:click=move |_| step(-1)
                    >
                        <Icon icon=id::FiChevronLeft width="22" height="22" />
                    </button>
                    <button
                        class="absolute top-1/2 right-1 -translate-y-1/2 rounded-full border-0 bg-black/55 p-2 text-white cursor-pointer"
                        aria-label="次の画像"
                        on:click=move |_| step(1)
                    >
                        <Icon icon=id::FiChevronRight width="22" height="22" />
                    </button>
                    <span class="wf-entry-meta">
                        {move || format!("{}/{}", index.get() + 1, len)}
                    </span>
                </Show>
            </div>
        </div>
    }
}
