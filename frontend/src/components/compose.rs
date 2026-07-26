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

    // 投稿送信
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
            file_ids: compose.file_ids.get_untracked(),
            reply_id: compose.reply_id.get_untracked(),
            poll_choices: compose.poll_choices.get_untracked(),
            scheduled_at: compose.scheduled_at.get_untracked(),
        };
        busy.set(true);
        error.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::notes::create_note(&tok, &request).await {
                Ok(_) => {
                    busy.set(false);
                    compose.clear();
                    compose.close();
                }
                Err(e) => {
                    busy.set(false);
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    let can_submit =
        move || !busy.get() && !compose.draft.get().trim().is_empty() && remaining.get() >= 0;

    view! {
        // ComposeA: centered wf-modal over wf-overlay
        <Show when=move || compose.is_open.get()>
            <div class="wf-overlay" on:click=move |_| compose.close()>
                <div
                    class="wf-modal"
                    style="max-width:540px;"
                    on:click=move |e| e.stop_propagation()
                >
                    // ヘッダー
                    <div class="wf-modal-head">
                        <div>
                            <p class="font-mono text-[10px] opacity-40 uppercase">"New Note"</p>
                            <h2 class="wf-modal-title">"投稿を書く"</h2>
                        </div>
                        <button
                            class="wf-btn wf-btn-ghost wf-btn-sm wf-btn-circle"
                            on:click=move |_| compose.close()
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                        </button>
                    </div>

                    <div class="wf-modal-body" style="display:flex;flex-direction:column;gap:12px;">
                        // 公開範囲 + CW
                        <div class="flex flex-wrap gap-3">
                            <select
                                class="wf-select"
                                on:change=move |event| {
                                    let value = event_target_value(&event);
                                    let visibility = match value.as_str() {
                                        "home" => NoteVisibility::Home,
                                        "followers" => NoteVisibility::Followers,
                                        "specified" => NoteVisibility::Specified,
                                        _ => NoteVisibility::Public,
                                    };
                                    compose.visibility.set(visibility);
                                }
                            >
                                <option value="public">"🌏 公開"</option>
                                <option value="home">"🏠 ホーム"</option>
                                <option value="followers">"🔒 フォロワー"</option>
                                <option value="specified">"✉️ 指定ユーザー"</option>
                            </select>
                            <input
                                class="wf-input"
                                style="flex:1;min-width:140px;"
                                placeholder="コンテンツ警告 (CW)"
                                prop:value=move || compose.cw.get()
                                on:input=move |event| compose.cw.set(event_target_value(&event))
                            />
                        </div>

                        // 返信先表示
                        <Show when=move || compose.reply_id.get().is_some()>
                            <div class="wf-cw">
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 14 4 9 9 4"/><path d="M20 20v-7a4 4 0 0 0-4-4H4"/></svg>
                                <strong>"返信中"</strong>
                                <button
                                    class="wf-btn wf-btn-ghost wf-btn-sm ml-auto"
                                    on:click=move |_| compose.reply_id.set(None)
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                                </button>
                            </div>
                        </Show>

                        // テキストエリア
                        <textarea
                            class="wf-compose-area"
                            maxlength="500"
                            placeholder="いま考えていることを書く… (Markdown 対応)"
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

                        // ファイル添付ドロップゾーン
                        <label
                            class="wf-drop flex items-center justify-center gap-2"
                            on:dragover=|e| {
                                e.prevent_default();
                            }
                            on:drop=move |e| {
                                e.prevent_default();
                                // TODO: Phase 5 でアップロード実装。現状は placeholder
                                let _ = e;
                            }
                        >
                            <input
                                type="file"
                                class="hidden"
                                multiple
                                accept="image/*,video/*"
                            />
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
                            <span>"画像・動画をここへドロップ (最大4ファイル・100MB)"</span>
                        </label>
                        // 選択済みファイル一覧
                        <Show when=move || !compose.file_ids.get().is_empty()>
                            <div class="flex flex-wrap gap-2">
                                {move || compose.file_ids.get().iter().enumerate().map(|(i, id)| {
                                    view! {
                                        <div class="wf-pill">
                                            <span>{id.chars().take(8).collect::<String>()}</span>
                                            <button
                                                class="opacity-50 hover:opacity-100"
                                                on:click=move |_| {
                                                    compose.file_ids.update(|ids| { ids.remove(i); });
                                                }
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                                            </button>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </Show>

                        // 投票選択肢
                        <div>
                            <button
                                class="wf-btn wf-btn-ghost wf-btn-sm"
                                on:click=move |_| {
                                    compose.poll_choices.update(|choices| choices.push(String::new()));
                                }
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
                                "投票を追加"
                            </button>
                            <Show when=move || !compose.poll_choices.get().is_empty()>
                                <div class="mt-2" style="display:flex;flex-direction:column;gap:6px;">
                                    {move || compose.poll_choices.get().iter().enumerate().map(|(i, _)| {
                                        view! {
                                            <div class="flex items-center gap-2">
                                                <input
                                                    class="wf-input"
                                                    style="flex:1;padding:6px 10px;font-size:13px;"
                                                    placeholder=format!("選択肢 {}", i+1)
                                                    prop:value=move || compose.poll_choices.get().get(i).cloned().unwrap_or_default()
                                                    on:input=move |event| {
                                                        let value = event_target_value(&event);
                                                        compose.poll_choices.update(|choices| {
                                                            if let Some(c) = choices.get_mut(i) { *c = value; }
                                                        });
                                                    }
                                                />
                                                <button
                                                    class="wf-btn wf-btn-ghost wf-btn-sm wf-btn-circle"
                                                    on:click=move |_| {
                                                        compose.poll_choices.update(|choices| { choices.remove(i); });
                                                    }
                                                >
                                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                                                </button>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </Show>
                        </div>

                        // 予約投稿
                        <Show when=move || compose.scheduled_at.get().is_some()>
                            <div class="wf-cw">
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                                <input
                                    type="datetime-local"
                                    class="wf-input"
                                    style="width:auto;padding:4px 8px;font-size:13px;"
                                    prop:value=move || compose.scheduled_at.get().unwrap_or_default()
                                    on:input=move |event| compose.scheduled_at.set(Some(event_target_value(&event)))
                                />
                                <button
                                    class="wf-btn wf-btn-ghost wf-btn-sm ml-auto"
                                    on:click=move |_| compose.scheduled_at.set(None)
                                >
                                    "キャンセル"
                                </button>
                            </div>
                        </Show>

                        // エラー
                        <Show when=move || error.get().is_some()>
                            <div class="wf-alert error">
                                <svg xmlns="http://www.w3.org/2000/svg" class="shrink-0" width="18" height="18" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                                {move || error.get().unwrap_or_default()}
                            </div>
                        </Show>
                    </div>

                    // フッター アクション
                    <div class="wf-modal-foot" style="justify-content:space-between;">
                        <div class="flex items-center gap-2">
                            <label class="flex items-center gap-2 text-sm cursor-pointer">
                                <input
                                    type="checkbox"
                                    class="wf-check"
                                    on:change=move |_| compose.nsfw.update(|v| *v = !*v)
                                />
                                <span class="font-mono text-xs">"NSFW"</span>
                            </label>
                            <button
                                class="wf-btn wf-btn-ghost wf-btn-sm"
                                on:click=move |_| {
                                    compose.scheduled_at.set(Some(String::new()));
                                }
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                                "予約"
                            </button>
                        </div>
                        <div class="flex items-center gap-3">
                            <span class=move || {
                                if remaining.get() < 0 {
                                    "font-mono text-xs font-bold"
                                } else if remaining.get() < 50 {
                                    "font-mono text-xs"
                                } else {
                                    "font-mono text-xs opacity-40"
                                }
                            } style=move || if remaining.get() < 0 { "color:var(--err)" } else if remaining.get() < 50 { "color:var(--warn)" } else { "" }>
                                {move || remaining.get().to_string()}
                            </span>
                            <button
                                class="wf-btn wf-btn-primary"
                                disabled=move || !can_submit()
                                on:click=move |_| submit()
                            >
                                {move || if busy.get() {
                                    view! { <span class="wf-spinner" style="width:16px;height:16px;border-width:2px;" /> }.into_any()
                                } else {
                                    view! { "⌘+Enter 投稿" }.into_any()
                                }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
