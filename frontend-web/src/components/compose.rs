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
        // DaisyUI modal
        <Show when=move || compose.is_open.get()>
            // backdrop
            <div
                class="fixed inset-0 bg-black/60 backdrop-blur-sm z-40"
                on:click=move |_| compose.close()
            />
            <div class="fixed inset-0 z-50 flex items-end sm:items-center justify-center p-0 sm:p-4">
                <div
                    class="modal-box w-full sm:max-w-xl rounded-t-2xl sm:rounded-2xl p-0 overflow-hidden shadow-2xl"
                    on:click=move |e| e.stop_propagation()
                >
                    // ヘッダー
                    <div class="flex items-center justify-between px-5 py-4 border-b border-base-300">
                        <div>
                            <p class="font-mono text-[10px] opacity-40 uppercase">"New Note"</p>
                            <h2 class="font-bold text-lg leading-tight">"投稿を書く"</h2>
                        </div>
                        <button
                            class="btn btn-ghost btn-sm btn-circle"
                            on:click=move |_| compose.close()
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                        </button>
                    </div>

                    // テキストエリア
                    <div class="px-5 pt-4 pb-2">
                        <textarea
                            class="compose-textarea"
                            maxlength="500"
                            placeholder="いま考えていることを書く… (MFM 対応)"
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
                    </div>

                    // オプション行
                    <div class="px-5 py-2 flex flex-wrap gap-3 border-t border-base-200">
                        // 公開範囲
                        <label class="form-control w-full sm:w-auto">
                            <select
                                class="select select-bordered select-sm"
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
                        </label>
                        // CW
                        <input
                            class="input input-bordered input-sm flex-1 min-w-[140px]"
                            placeholder="コンテンツ警告 (CW)"
                            prop:value=move || compose.cw.get()
                            on:input=move |event| compose.cw.set(event_target_value(&event))
                        />
                    </div>

                    // 返信先表示
                    <Show when=move || compose.reply_id.get().is_some()>
                        <div class="px-5 py-2 flex items-center gap-2 text-sm opacity-60 border-b border-base-200">
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 14 4 9 9 4"/><path d="M20 20v-7a4 4 0 0 0-4-4H4"/></svg>
                            <span>"返信中"</span>
                            <button
                                class="btn btn-ghost btn-xs ml-auto"
                                on:click=move |_| compose.reply_id.set(None)
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                            </button>
                        </div>
                    </Show>

                    // ファイル添付ドロップゾーン
                    <div class="px-5 py-2">
                        <label
                            class="drop-zone flex items-center justify-center gap-2 cursor-pointer"
                            on:dragover=|e| {
                                e.prevent_default();
                            }
                            on:drop=move |e| {
                                e.prevent_default();
                                if let Some(dt) = e.data_transfer() {
                                    if let Some(files) = dt.files() {
                                        // TODO: Phase 5 でアップロード実装。現状は placeholder
                                        let _count = files.length();
                                    }
                                }
                            }
                        >
                            <input
                                type="file"
                                class="hidden"
                                multiple
                                accept="image/*,video/*"
                            />
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
                            <span class="text-sm">"画像・動画をここへドロップ (最大4ファイル・100MB)"</span>
                        </label>
                        // 選択済みファイル一覧
                        <Show when=move || !compose.file_ids.get().is_empty()>
                            <div class="flex flex-wrap gap-2 mt-2">
                                {move || compose.file_ids.get().iter().enumerate().map(|(i, id)| {
                                    view! {
                                        <div class="badge badge-ghost gap-1">
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
                    </div>

                    // 投票選択肢
                    <div class="px-5 py-2 border-t border-base-200">
                        <button
                            class="btn btn-ghost btn-xs"
                            on:click=move |_| {
                                compose.poll_choices.update(|choices| choices.push(String::new()));
                            }
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
                            "投票を追加"
                        </button>
                        <Show when=move || !compose.poll_choices.get().is_empty()>
                            <div class="mt-2 space-y-1">
                                {move || compose.poll_choices.get().iter().enumerate().map(|(i, _)| {
                                    view! {
                                        <div class="flex items-center gap-2">
                                            <input
                                                class="input input-bordered input-xs flex-1"
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
                                                class="btn btn-ghost btn-xs btn-square"
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
                        <div class="px-5 py-2 flex items-center gap-2 text-sm border-t border-base-200">
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                            <input
                                type="datetime-local"
                                class="input input-bordered input-xs"
                                prop:value=move || compose.scheduled_at.get().unwrap_or_default()
                                on:input=move |event| compose.scheduled_at.set(Some(event_target_value(&event)))
                            />
                            <button
                                class="btn btn-ghost btn-xs ml-auto"
                                on:click=move |_| compose.scheduled_at.set(None)
                            >
                                "キャンセル"
                            </button>
                        </div>
                    </Show>

                    // エラー
                    <Show when=move || error.get().is_some()>
                        <div class="alert alert-error mx-5 py-2 text-sm">
                            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-5 w-5" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>

                    // フッター アクション
                    <div class="flex items-center justify-between px-5 py-4 border-t border-base-300 bg-base-200/50">
                        <div class="flex items-center gap-2">
                            <label class="label cursor-pointer gap-2 text-sm">
                                <input
                                    type="checkbox"
                                    class="checkbox checkbox-sm checkbox-error"
                                    on:change=move |_| compose.nsfw.update(|v| *v = !*v)
                                />
                                <span class="label-text font-mono text-xs">"NSFW"</span>
                            </label>
                            <button
                                class="btn btn-ghost btn-xs"
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
                                    "font-mono text-xs text-error font-bold"
                                } else if remaining.get() < 50 {
                                    "font-mono text-xs text-warning"
                                } else {
                                    "font-mono text-xs opacity-40"
                                }
                            }>
                                {move || remaining.get().to_string()}
                            </span>
                            <button
                                class="btn btn-primary rounded-full px-6"
                                disabled=move || !can_submit()
                                on:click=move |_| submit()
                            >
                                {move || if busy.get() {
                                    view! { <span class="loading loading-spinner loading-sm" /> }.into_any()
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
