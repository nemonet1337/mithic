use leptos::prelude::*;

use crate::models::{CreateNoteRequest, NoteVisibility};
use crate::store::{AuthStore, ComposeStore};

#[component]
pub fn ComposeModal() -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    let auth = expect_context::<AuthStore>();
    let token = auth.token;
    let busy = RwSignal::new(false);
    let error = RwSignal::<Option<String>>::new(None);
    let remaining = Memo::new(move |_| 500isize - compose.draft.get().chars().count() as isize);

    // 投稿送信 (F-3: ComposeModal 実投稿)
    let submit = move || {
        if busy.get_untracked() {
            return;
        }
        let Some(tok) = token.get_untracked() else {
            error.set(Some("ログインが必要です".into()));
            return;
        };
        let text = compose.draft.get_untracked();
        if text.trim().is_empty() || text.chars().count() > 500 {
            return;
        }
        let cw = compose.cw.get_untracked();
        let request = CreateNoteRequest {
            text,
            visibility: compose.visibility.get_untracked(),
            cw: if cw.trim().is_empty() { None } else { Some(cw) },
            is_nsfw: compose.nsfw.get_untracked(),
            file_ids: Vec::new(),
            reply_id: None,
            poll_choices: Vec::new(),
            scheduled_at: None,
        };
        busy.set(true);
        error.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::notes::create_note(&tok, &request).await {
                Ok(_) => {
                    busy.set(false);
                    // 成功: 下書きを消してモーダルを閉じる。
                    // タイムラインへの差し込みは WebSocket ストリーム経由で行われる。
                    compose.clear();
                    compose.close();
                }
                Err(e) => {
                    busy.set(false);
                    // 失敗: 入力は保持したままエラー表示
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    let can_submit =
        move || !busy.get() && !compose.draft.get().trim().is_empty() && remaining.get() >= 0;

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
                        on:keydown=move |event| {
                            if event.key() == "Enter" && (event.ctrl_key() || event.meta_key()) {
                                event.prevent_default();
                                submit();
                            }
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
                    <Show when=move || error.get().is_some()>
                        <div class="auth-error">
                            <span class="wf-pill accent" style="font-size:9px">"[ ERROR ]"</span>
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    <div class="compose-options">
                        <label class="wf-pill"><input type="checkbox" on:change=move |_| compose.nsfw.update(|value| *value = !*value) />" NSFW"</label>
                        <button class="wf-btn sm ghost">"投票 +"</button>
                        <button class="wf-btn sm ghost">"絵文字"</button>
                        <button class="wf-btn sm ghost">"予約"</button>
                        <span class=move || if remaining.get() < 0 { "wf-pill accent" } else { "wf-pill" }>{move || remaining.get().to_string()}</span>
                    </div>
                    <div class="wf-spread compose-actions">
                        <button class="wf-btn ghost" on:click=move |_| compose.close()>"Esc 閉じる"</button>
                        <button class="wf-btn accent" disabled=move || !can_submit() on:click=move |_| submit()>
                            {move || if busy.get() { "送信中…" } else { "⌘Enter 投稿" }}
                        </button>
                    </div>
                </section>
            </div>
        </Show>
    }
}
