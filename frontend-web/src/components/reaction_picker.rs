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
            cb.run(emoji.to_string());
        }
        if let Some(ref cb) = on_close {
            cb.run(());
        }
    };

    view! {
        <Show when=move || is_open>
            <div class="wf-pop" style="width:280px;">
                <div class="wf-react-key">
                    {COMMON_EMOJIS.iter().map(|emoji| {
                        view! {
                            <button
                                on:click=move |_| select_emoji(emoji)
                            >
                                {emoji.to_string()}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
                <div class="flex gap-2" style="padding:0 4px 4px;">
                    <input
                        class="wf-input"
                        style="padding:6px 10px;font-size:12px;"
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
                        class="wf-btn wf-btn-ghost wf-btn-sm"
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
