use leptos::prelude::*;

const COMMON_EMOJIS: &[&str] = &[
    "🔥", "✨", "👍", "❤️", "🎉", "😊", "😂", "😢", "😡", "💯",
    "⭐", "🌟", "💖", "🎨", "🔧", "🚀", "💡", "📝", "🎵", "🌈",
];

#[component]
pub fn ReactionPicker(
    #[prop(default = None)]
    on_select: Option<Callback<String>>,
    #[prop(default = false)]
    is_open: bool,
    #[prop(default = None)]
    on_close: Option<Callback<()>>,
) -> impl IntoView {
    let custom_input = RwSignal::new(String::new());

    let select_emoji = move |emoji: &str| {
        if let Some(ref cb) = on_select {
            cb.call(emoji.to_string());
        }
        if let Some(ref cb) = on_close {
            cb.call(());
        }
    };

    view! {
        <Show when=move || is_open>
            <div class="absolute z-50 mt-1 p-3 bg-base-100 border border-base-300 rounded-2xl shadow-xl w-72">
                <div class="grid grid-cols-10 gap-1 mb-2">
                    {COMMON_EMOJIS.iter().map(|emoji| {
                        view! {
                            <button
                                class="btn btn-ghost btn-xs p-1 text-lg hover:bg-base-200 rounded-lg"
                                on:click=move |_| select_emoji(emoji)
                            >
                                {emoji.to_string()}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
                <div class="flex gap-2">
                    <input
                        class="input input-bordered input-xs flex-1"
                        placeholder="カスタム絵文字 :emoji:"
                        prop:value=move || custom_input.get()
                        on:input=move |e| custom_input.set(event_target_value(&e))
                        on:keydown=move |e| {
                            if e.key() == "Enter" {
                                let val = custom_input.get_untracked();
                                if !val.is_empty() {
                                    select_emoji(&format!(":{}:", val));
                                }
                            }
                        }
                    />
                    <button
                        class="btn btn-xs btn-ghost"
                        on:click=move |_| {
                            let val = custom_input.get_untracked();
                            if !val.is_empty() {
                                select_emoji(&format!(":{}:", val));
                            }
                        }
                    >
                        "追加"
                    </button>
                </div>
            </div>
        </Show>
    }
}
