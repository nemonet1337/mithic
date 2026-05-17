use leptos::prelude::*;

use crate::models::NoteVisibility;
use crate::store::ComposeStore;

#[component]
pub fn ComposeModal() -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    let remaining = Memo::new(move |_| 500isize - compose.draft.get().chars().count() as isize);

    view! {
        <Show when=move || compose.is_open.get()>
            <div class="compose-backdrop" on:click=move |_| compose.close()>
                <section class="wf-card raised compose-modal" on:click=move |event| event.stop_propagation()>
                    <div class="wf-spread compose-head">
                        <div>
                            <span class="wf-label">"[ NEW NOTE ]"</span>
                            <h2 class="wf-hand compose-title">"投稿を書く"</h2>
                        </div>
                        <button class="wf-btn icon ghost" on:click=move |_| compose.close()>"×"</button>
                    </div>
                    <textarea
                        class="compose-textarea"
                        maxlength="500"
                        placeholder="いま考えていることをMFMで書く…"
                        prop:value=move || compose.draft.get()
                        on:input=move |event| {
                            compose.draft.set(event_target_value(&event));
                            compose.save_draft();
                        }
                    />
                    <div class="compose-grid">
                        <label class="compose-field">
                            <span class="wf-label">"公開範囲"</span>
                            <select on:change=move |event| {
                                let value = event_target_value(&event);
                                let visibility = match value.as_str() {
                                    "home" => NoteVisibility::Home,
                                    "followers" => NoteVisibility::Followers,
                                    "specified" => NoteVisibility::Specified,
                                    _ => NoteVisibility::Public,
                                };
                                compose.visibility.set(visibility);
                            }>
                                <option value="public">"公開"</option>
                                <option value="home">"ホーム"</option>
                                <option value="followers">"フォロワー"</option>
                                <option value="specified">"指定ユーザー"</option>
                            </select>
                        </label>
                        <label class="compose-field">
                            <span class="wf-label">"CW"</span>
                            <input
                                placeholder="コンテンツ警告"
                                prop:value=move || compose.cw.get()
                                on:input=move |event| compose.cw.set(event_target_value(&event))
                            />
                        </label>
                    </div>
                    <div class="wf-card dashed drop-zone">
                        <span class="wf-mono">"画像・動画をここへドロップ / 最大4ファイル・100MB"</span>
                    </div>
                    <div class="compose-options">
                        <label class="wf-pill"><input type="checkbox" on:change=move |_| compose.nsfw.update(|value| *value = !*value) />" NSFW"</label>
                        <button class="wf-btn sm ghost">"投票 +"</button>
                        <button class="wf-btn sm ghost">"絵文字"</button>
                        <button class="wf-btn sm ghost">"予約"</button>
                        <span class=move || if remaining.get() < 0 { "wf-pill accent" } else { "wf-pill" }>{move || remaining.get().to_string()}</span>
                    </div>
                    <div class="wf-spread compose-actions">
                        <button class="wf-btn ghost" on:click=move |_| compose.close()>"Esc 閉じる"</button>
                        <button class="wf-btn accent" disabled=move || compose.draft.get().trim().is_empty() || remaining.get() < 0 on:click=move |_| {
                            compose.clear();
                            compose.close();
                        }>
                            "⌘Enter 投稿"
                        </button>
                    </div>
                </section>
            </div>
        </Show>
    }
}
