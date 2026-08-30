use leptos::prelude::*;

const GROUPS: &[(&str, &[&str])] = &[
    (
        "よく使う",
        &["🔥", "✨", "👀", "💬", "🌱", "☕", "📚", "🎨"],
    ),
    ("感情", &["😊", "😂", "🥺", "😎", "🤔", "😴", "💖", "🙏"]),
    ("反応", &["👍", "👎", "👏", "🙌", "✋", "👋", "🤝", "💪"]),
];

#[component]
pub fn ReactionPicker(
    #[prop(into)] is_open: Signal<bool>,
    #[prop(optional)] on_select: Option<Callback<String>>,
    #[prop(optional)] on_close: Option<Callback<()>>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let custom_input = RwSignal::new(String::new());

    let select_emoji = move |emoji: String| {
        if let Some(ref cb) = on_select {
            cb.run(emoji);
        }
        if let Some(ref cb) = on_close {
            cb.run(());
        }
        query.set(String::new());
        custom_input.set(String::new());
    };

    view! {
        <Show when=move || is_open.get()>
            <div class="wf-pop wf-react-pop" style="width:320px;padding:10px;">
                <div class="flex items-center justify-between mb-2">
                    <span class="wf-entry-meta">"[ リアクション ]"</span>
                    <button
                        class="wf-btn wf-btn-ghost wf-btn-sm wf-btn-circle"
                        on:click=move |_| {
                            if let Some(ref cb) = on_close {
                                cb.run(());
                            }
                        }
                    >
                        "×"
                    </button>
                </div>
                <input
                    class="wf-input mb-2"
                    style="height:32px;padding:6px 10px;font-size:12px;"
                    placeholder="絵文字を検索…"
                    prop:value=move || query.get()
                    on:input=move |e| query.set(event_target_value(&e))
                />
                {GROUPS.iter().map(|(label, items)| {
                    let label = *label;
                    let items = *items;
                    view! {
                        <div>
                            <span class="wf-entry-meta uppercase">{label}</span>
                            <div class="wf-react-key" style="padding:4px 0 8px;">
                                {move || {
                                    let q = query.get();
                                    items.iter().filter(|e| q.is_empty() || e.contains(&q)).map(|emoji| {
                                        let owned = emoji.to_string();
                                        let display = *emoji;
                                        view! {
                                            <button on:click=move |_| select_emoji(owned.clone())>
                                                {display}
                                            </button>
                                        }
                                    }).collect_view()
                                }}
                            </div>
                        </div>
                    }
                }).collect_view()}
                <div class="flex gap-2 mt-1 pt-2" style="border-top:1px dashed var(--line-soft);">
                    <input
                        class="wf-input"
                        style="padding:6px 10px;font-size:12px;"
                        placeholder="カスタム :emoji:"
                        prop:value=move || custom_input.get()
                        on:input=move |e| custom_input.set(event_target_value(&e))
                        on:keydown=move |e| {
                            if e.key() == "Enter" {
                                let val = custom_input.get_untracked();
                                if !val.is_empty() {
                                    select_emoji(normalize_custom(&val));
                                }
                            }
                        }
                    />
                    <button
                        class="wf-btn wf-btn-ghost wf-btn-sm"
                        on:click=move |_| {
                            let val = custom_input.get_untracked();
                            if !val.is_empty() {
                                select_emoji(normalize_custom(&val));
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

fn normalize_custom(raw: &str) -> String {
    let t = raw.trim();
    if t.starts_with(':') {
        t.to_string()
    } else {
        format!(":{t}:")
    }
}
