use leptos::prelude::*;

const COMMON_EMOJIS: &[&str] = &[
    "🔥", "✨", "👍", "❤️", "🎉", "😊", "😂", "😢", "😡", "💯", "⭐", "🌟", "💖", "🎨", "🔧",
    "🚀", "💡", "📝", "🎵", "🌈",
];

#[component]
pub fn ReactionPicker(
    #[prop(into)] is_open: Signal<bool>,
    #[prop(optional)] on_select: Option<Callback<String>>,
    #[prop(optional)] on_close: Option<Callback<()>>,
) -> impl IntoView {
    let custom_input = RwSignal::new(String::new());

    let select_emoji = move |emoji: String| {
        if let Some(ref cb) = on_select {
            cb.run(emoji);
        }
        if let Some(ref cb) = on_close {
            cb.run(());
        }
    };

    view! {
        <Show when=move || is_open.get()>
            <div class="wf-pop" style="width:280px;">
                <div class="wf-react-key">
                    {COMMON_EMOJIS.iter().map(|&emoji| {
                        let emoji_owned = emoji.to_string();
                        let emoji_display = emoji.to_string();
                        view! {
                            <button
                                on:click=move |_| select_emoji(emoji_owned.clone())
                            >
                                {emoji_display}
                            </button>
                        }
                    }).collect_view()}
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
                                    select_emoji(format!(":{val}:"));
                                }
                            }
                        }
                    />
                    <button
                        class="wf-btn wf-btn-ghost wf-btn-sm"
                        on:click=move |_| {
                            let val = custom_input.get_untracked();
                            if !val.is_empty() {
                                select_emoji(format!(":{val}:"));
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
