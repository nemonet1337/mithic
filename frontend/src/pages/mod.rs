use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::components::{
    Avatar, AvatarSize, ConfirmDialog, FollowButton, NotificationsColumn, PostCard, SearchColumn,
    Shell, TimelineColumn, TimelineKind, ToastKind, ToastStore, TopBar,
};
use crate::store::{AuthStore, StreamStore};
use shared::{Note, User};

mod settings;
pub use settings::SettingsPage;

mod drive;
pub use drive::DrivePage;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Shell active="home">
            <TimelineColumn kind=TimelineKind::Home />
        </Shell>
    }
}

#[component]
pub fn LocalTimelinePage() -> impl IntoView {
    view! {
        <Shell active="local">
            <TimelineColumn kind=TimelineKind::Local />
        </Shell>
    }
}

#[component]
pub fn GlobalTimelinePage() -> impl IntoView {
    view! {
        <Shell active="global">
            <TimelineColumn kind=TimelineKind::Global />
        </Shell>
    }
}

#[component]
pub fn StatusDetailPage() -> impl IntoView {
    let params = use_params_map();
    let auth = expect_context::<AuthStore>();
    let stream = expect_context::<StreamStore>();
    let note = RwSignal::<Option<Note>>::new(None);
    let ancestors = RwSignal::<Vec<Note>>::new(Vec::new());
    let replies = RwSignal::<Vec<Note>>::new(Vec::new());
    let error = RwSignal::<Option<String>>::new(None);
    let loading = RwSignal::new(true);

    Effect::new(move |_| {
        let id = params.read().get("id").unwrap_or_default();
        let Some(tok) = auth.token.get() else {
            loading.set(false);
            error.set(Some("ログインが必要です".into()));
            return;
        };
        if id.is_empty() {
            loading.set(false);
            error.set(Some("投稿 ID がありません".into()));
            return;
        }
        loading.set(true);
        error.set(None);
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::notes::fetch_note(&tok, &id).await {
                Ok(n) => {
                    ancestors.set(Vec::new());
                    note.set(Some(n));
                    match crate::api::notes::fetch_replies(&tok, &id).await {
                        Ok(r) => replies.set(r),
                        Err(e) => web_sys::console::error_1(&e.to_string().into()),
                    }
                }
                Err(e) => error.set(Some(e.user_message())),
            }
            loading.set(false);
        });
    });

    // reply_id を辿って親を遡り、会話の流れを上で見せる
    Effect::new(move |_| {
        let Some(current) = note.get() else {
            return;
        };
        let Some(parent_id) = current.reply_id.clone() else {
            return;
        };
        wasm_bindgen_futures::spawn_local(async move {
            let Some(tok) = auth.token.get() else {
                return;
            };
            let Ok(parent) = crate::api::notes::fetch_note(&tok, &parent_id).await else {
                return;
            };
            let mut chain = Vec::new();
            if let Some(grand) = parent.reply_id.clone()
                && let Ok(grandparent) = crate::api::notes::fetch_note(&tok, &grand).await
            {
                chain.push(grandparent);
            }
            chain.push(parent);
            ancestors.set(chain);
        });
    });

    let has_ancestors = Signal::derive(move || !ancestors.get().is_empty());

    view! {
        <Shell active="home">
            <div class="flex items-center gap-2 px-4 pt-3">
                <A href="/" attr:class="wf-btn wf-btn-ghost wf-btn-sm">"← Esc"</A>
                <span class="wf-entry-meta ml-auto">{move || format!("POST · /{}", params.read().get("id").unwrap_or_default())}</span>
            </div>
            <div class="wf-scroll">
                <Show when=move || loading.get()>
                    <div class="flex items-center justify-center gap-2 py-8">
                        <span class="wf-spinner" style="width:18px;height:18px;" />
                        <span class="wf-entry-meta">"読み込み中…"</span>
                    </div>
                </Show>
                <Show when=move || error.get().is_some()>
                    <div class="wf-alert error m-4">
                        <span>{move || error.get().unwrap_or_default()}</span>
                    </div>
                </Show>
                {move || {
                    let deleted = note.get().is_some_and(|n| stream.deleted_ids.get().contains(&n.id));
                    if deleted {
                        return view! {
                            <div class="wf-empty m-4">
                                <span>"この投稿は削除されました"</span>
                            </div>
                        }.into_any();
                    }
                    note.get().map(|current| {
                    let reactions = current.reactions.clone();
                    view! {
                        <div class="wf-detail-split">
                            <div class="flex flex-col gap-3">
                                <Show when=move || has_ancestors.get()>
                                    <span class="wf-entry-meta">"[ 会話 / CONTEXT ]"</span>
                                    <For
                                        each=move || ancestors.get()
                                        key=|n| n.id.clone()
                                        children=|n| view! { <PostCard note=n /> }
                                    />
                                    <hr class="wf-rule" />
                                </Show>
                                <PostCard note=current.clone() />
                                <span class="wf-entry-meta">"[ 返信 / REPLIES ]"</span>
                                <Show when=move || replies.get().is_empty()>
                                    <div class="wf-dashed p-6 text-center">
                                        <span class="wf-entry-meta">"まだ返信はありません"</span>
                                    </div>
                                </Show>
                                <For
                                    each=move || stream.visible(replies.get())
                                    key=|n| n.id.clone()
                                    children=|n| view! { <PostCard note=n /> }
                                />
                            </div>
                            <aside class="wf-card h-fit">
                                <span class="wf-entry-meta">"[ リアクション ]"</span>
                                <div class="flex flex-wrap gap-2 mt-3">
                                    {if reactions.is_empty() {
                                        view! { <span class="wf-entry-meta">"まだありません"</span> }.into_any()
                                    } else {
                                        reactions.iter().filter(|r| r.count > 0).map(|r| {
                                            let label = format!("{} {}", r.emoji, r.count);
                                            view! { <span class=if r.reacted_by_me { "wf-pill on" } else { "wf-pill" }>{label}</span> }
                                        }).collect_view().into_any()
                                    }}
                                </div>
                            </aside>
                        </div>
                    }
                    }).into_any()
                }}
            </div>
        </Shell>
    }
}

#[component]
pub fn NotificationsPage() -> impl IntoView {
    view! {
        <Shell active="notif">
            <NotificationsColumn />
        </Shell>
    }
}

#[component]
pub fn SearchPage() -> impl IntoView {
    view! {
        <Shell active="search">
            <SearchColumn />
        </Shell>
    }
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params_map();
    let auth = expect_context::<AuthStore>();
    let token = auth.token;
    let handle = move || {
        params
            .read()
            .get("username")
            .unwrap_or_else(|| "hana".into())
    };
    let user = RwSignal::<Option<User>>::new(None);
    let notes = RwSignal::<Vec<Note>>::new(vec![]);
    let is_following = RwSignal::new(false);
    let is_follow_requested = RwSignal::new(false);
    let is_blocking = RwSignal::new(false);
    let is_muted = RwSignal::new(false);
    let follow_busy = RwSignal::new(false);
    let profile_tab = RwSignal::new("notes");
    let stream = expect_context::<StreamStore>();

    // プロフィールと投稿一覧を実 API から取得
    Effect::new(move |_| {
        let username = handle();
        if let Some(tok) = token.get() {
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::users::fetch_user(&tok, &username).await {
                    Ok(fetched) => {
                        let id = fetched.id.clone();
                        user.set(Some(fetched));
                        if let Ok(rel) = crate::api::users::fetch_relation(&tok, &id).await {
                            is_following.set(rel.is_following);
                            is_follow_requested.set(rel.is_follow_requested);
                            is_blocking.set(rel.is_blocking);
                            is_muted.set(rel.is_muted);
                        }
                    }
                    Err(e) => web_sys::console::error_1(&e.to_string().into()),
                }
                match crate::api::users::fetch_user_notes(&tok, &username).await {
                    Ok(fetched) => notes.set(fetched),
                    Err(e) => web_sys::console::error_1(&e.to_string().into()),
                }
            });
        }
    });

    let toast = expect_context::<ToastStore>();
    let toggle_follow = Callback::new(move |_: ()| {
        if follow_busy.get_untracked() {
            return;
        }
        let (Some(tok), Some(target)) = (token.get_untracked(), user.get_untracked()) else {
            return;
        };
        follow_busy.set(true);
        let currently = is_following.get_untracked() || is_follow_requested.get_untracked();
        let toast = toast;
        wasm_bindgen_futures::spawn_local(async move {
            if currently {
                match crate::api::users::unfollow(&tok, &target.id).await {
                    Ok(()) => {
                        is_following.set(false);
                        is_follow_requested.set(false);
                        toast.push("フォローを解除しました", ToastKind::Success);
                    }
                    Err(e) => toast.push(e.user_message(), ToastKind::Error),
                }
            } else {
                match crate::api::users::follow(&tok, &target.id).await {
                    Ok(res) => {
                        is_following.set(!res.is_pending);
                        is_follow_requested.set(res.is_pending);
                        if res.is_pending {
                            toast.push("フォローリクエストを送信しました", ToastKind::Info);
                        } else if let Some(msg) = res.followed_message.filter(|m| !m.is_empty()) {
                            toast.push(msg, ToastKind::Info);
                        } else {
                            toast.push("フォローしました", ToastKind::Success);
                        }
                    }
                    Err(e) => toast.push(e.user_message(), ToastKind::Error),
                }
            }
            follow_busy.set(false);
        });
    });

    // ブロック / ミュートは確認ダイアログ経由
    let relation_action = RwSignal::new("");
    let run_relation_action = Callback::new(move |action: &'static str| {
        let (Some(tok), Some(target)) = (token.get_untracked(), user.get_untracked()) else {
            return;
        };
        let toast = toast;
        wasm_bindgen_futures::spawn_local(async move {
            let result = match action {
                "block" => crate::api::users::block(&tok, &target.id).await,
                "unblock" => crate::api::users::unblock(&tok, &target.id).await,
                "mute" => crate::api::users::mute(&tok, &target.id).await,
                _ => crate::api::users::unmute(&tok, &target.id).await,
            };
            match result {
                Ok(rel) => {
                    is_blocking.set(rel.is_blocking);
                    is_muted.set(rel.is_muted);
                    toast.push(
                        action_label(action, rel.is_blocking, rel.is_muted),
                        ToastKind::Success,
                    );
                }
                Err(e) => toast.push(e.user_message(), ToastKind::Error),
            }
        });
    });

    view! {
        <Shell active="profile">
            <section class="wf-scroll wf-profile-editorial">
                <div
                    class="wf-profile-banner"
                    style=move || user.get().and_then(|u| u.banner_url).map(|url| format!("background-image:url('{url}');background-size:cover;background-position:center;"))
                />
                <div class="px-6 pt-5 pb-2">
                    <span class="wf-entry-meta">"PROFILE"</span>
                    <div class="flex items-start justify-between gap-4 mt-1">
                        <div class="min-w-0">
                            <h1 class="wf-profile-name">{move || user.get().map(|u| u.name()).unwrap_or_default()}</h1>
                            <div class="flex flex-wrap gap-2 mt-3">
                                <span class="wf-pill">{move || format!("@{}", handle())}</span>
                                {move || user.get().and_then(|u| u.location).map(|loc| view! { <span class="wf-pill">{format!("📍 {loc}")}</span> })}
                                {move || user.get().and_then(|u| u.birthday).map(|b| view! { <span class="wf-pill">{format!("🎂 {b}")}</span> })}
                                {move || user.get().filter(|u| u.is_bot).map(|_| view! { <span class="wf-pill">"Bot"</span> })}
                                {move || user.get().filter(|u| u.is_cat).map(|_| view! { <span class="wf-pill on">"cat"</span> })}
                            </div>
                        </div>
                        {move || user.get().map(|u| view! { <Avatar user=u size=AvatarSize::Xl /> })}
                    </div>
                    <div class="flex flex-wrap items-center gap-2 mt-3">
                        <Show when=move || auth.me.get().zip(user.get()).map(|(me, u)| me.id != u.id).unwrap_or(false)>
                            <FollowButton
                                is_following=is_following
                                is_pending=follow_busy
                                is_requested=is_follow_requested
                                on_toggle=toggle_follow
                            />
                            <button
                                class="wf-btn wf-btn-ghost wf-btn-sm"
                                on:click={
                                    let action = relation_action;
                                    move |_| action.set(if is_blocking.get_untracked() { "unblock" } else { "block" })
                                }
                            >
                                {move || if is_blocking.get() { "ブロック解除" } else { "ブロック" }}
                            </button>
                            <button
                                class="wf-btn wf-btn-ghost wf-btn-sm"
                                on:click={
                                    let action = relation_action;
                                    move |_| action.set(if is_muted.get_untracked() { "unmute" } else { "mute" })
                                }
                            >
                                {move || if is_muted.get() { "ミュート解除" } else { "ミュート" }}
                            </button>
                        </Show>
                    </div>
                    <div class="wf-profile-grid">
                        <div>
                            <span class="wf-entry-meta">"BIO"</span>
                            <p class="text-sm mt-2 leading-relaxed">{move || user.get().and_then(|u| u.bio).unwrap_or_default()}</p>
                        </div>
                        <div>
                            <span class="wf-entry-meta">"追加情報"</span>
                            <div class="flex flex-col gap-1 mt-2">
                                {move || {
                                    let fields = user.get().map(|u| u.fields).unwrap_or_default();
                                    if fields.is_empty() {
                                        view! { <span class="wf-entry-meta">"—"</span> }.into_any()
                                    } else {
                                        fields.into_iter().map(|f| {
                                            view! {
                                                <div class="wf-spread text-sm">
                                                    <span class="text-ink-soft">{f.name}</span>
                                                    <span>{f.value}</span>
                                                </div>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }}
                            </div>
                        </div>
                        <div>
                            <span class="wf-entry-meta">"指標"</span>
                            <div class="flex flex-col gap-1 mt-2 text-sm">
                                <div class="wf-spread"><span>"投稿"</span><b class="font-mono">{move || user.get().map(|u| u.notes_count).unwrap_or(0).to_string()}</b></div>
                                <div class="wf-spread"><span>"フォロワー"</span><b class="font-mono">{move || user.get().map(|u| u.followers_count).unwrap_or(0).to_string()}</b></div>
                                <div class="wf-spread"><span>"フォロー"</span><b class="font-mono">{move || user.get().map(|u| u.following_count).unwrap_or(0).to_string()}</b></div>
                                <div class="wf-spread"><span>"参加"</span><span class="font-mono text-xs">{move || user.get().and_then(|u| u.created_at).map(|c| crate::time::date_label(&c)).unwrap_or_default()}</span></div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="wf-profile-tabs">
                    <a
                        class=move || if profile_tab.get() == "notes" { "active" } else { "" }
                        on:click=move |_| profile_tab.set("notes")
                        style="cursor:pointer">
                        "投稿"
                    </a>
                    <a
                        class=move || if profile_tab.get() == "media" { "active" } else { "" }
                        on:click=move |_| profile_tab.set("media")
                        style="cursor:pointer">
                        "メディア"
                    </a>
                </div>

                <div class="flex flex-col gap-3 px-4 mt-3">
                    {move || match profile_tab.get() {
                        "media" => {
                            let media_notes: Vec<_> = notes
                                .get()
                                .into_iter()
                                .filter(|n| !n.attachments.is_empty())
                                .collect();
                            if media_notes.is_empty() {
                                view! {
                                    <div class="wf-empty">
                                        <span>"まだメディアがありません"</span>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <For
                                        each=move || {
                                            notes
                                                .get()
                                                .into_iter()
                                                .filter(|n| !n.attachments.is_empty())
                                                .collect::<Vec<_>>()
                                        }
                                        key=|note| note.id.clone()
                                        children=|note| view! { <PostCard note=note /> }
                                    />
                                }.into_any()
                            }
                        }
                        _ => {
                            let list = notes.get();
                            if list.is_empty() {
                                view! {
                                    <div class="wf-empty">
                                        <span>"まだ投稿がありません"</span>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <For
                                        each=move || stream.visible(notes.get())
                                        key=|note| note.id.clone()
                                        children=|note| view! { <PostCard note=note /> }
                                    />
                                }.into_any()
                            }
                        }
                    }}
                </div>
            </section>
            <ConfirmDialog
                is_open=Signal::derive(move || relation_action.get() != "")
                title=Signal::derive(move || action_title(relation_action.get()))
                body=Signal::derive(move || action_body(relation_action.get()))
                preview_meta="対象アカウント"
                preview=Signal::derive(move || user.get().map(|u| u.handle()).unwrap_or_default())
                confirm_label=Signal::derive(move || action_confirm_label(relation_action.get()))
                danger=Signal::derive(move || matches!(relation_action.get(), "block" | "unblock"))
                on_confirm=Callback::new({
                    let run = run_relation_action;
                    move |()| run.run(leak_action(relation_action.get()))
                })
                on_close=Callback::new({
                    let action = relation_action;
                    move |()| action.set("")
                })
            />
        </Shell>
    }
}

fn leak_action(action: &str) -> &'static str {
    match action {
        "block" => "block",
        "unblock" => "unblock",
        "mute" => "mute",
        _ => "unmute",
    }
}

fn action_title(action: &str) -> String {
    match action {
        "block" => "ブロックしますか？".into(),
        "unblock" => "ブロックを解除しますか？".into(),
        "mute" => "ミュートしますか？".into(),
        "unmute" => "ミュートを解除しますか？".into(),
        _ => String::new(),
    }
}

fn action_body(action: &str) -> String {
    match action {
        "block" => {
            "ブロックすると、このアカウントの投稿はタイムラインに表示されなくなります。".into()
        }
        "unblock" => "ブロックを解除すると、このアカウントの投稿が再び表示されます。".into(),
        "mute" => "ミュートすると、タイムラインに表示されなくなります（通知は届きます）。".into(),
        "unmute" => "ミュートを解除すると、タイムラインに再び表示されます。".into(),
        _ => String::new(),
    }
}

fn action_confirm_label(action: &str) -> String {
    match action {
        "block" => "ブロックする",
        "unblock" => "解除する",
        "mute" => "ミュートする",
        "unmute" => "解除する",
        _ => "実行する",
    }
    .into()
}

fn action_label(action: &str, blocking: bool, muted: bool) -> String {
    match action {
        "block" | "unblock" => {
            if blocking {
                "ブロックしました".into()
            } else {
                "ブロックを解除しました".into()
            }
        }
        _ => {
            if muted {
                "ミュートしました".into()
            } else {
                "ミュートを解除しました".into()
            }
        }
    }
}

/// Login / Signup 共通の 2 カラム auth 枠
#[component]
fn AuthShell(
    #[prop(into)] kicker: String,
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="wf-auth">
            <aside class="wf-auth-aside">
                <span class="wf-mark wf-mark-lg">"[m]"<span class="br">"mithic"</span></span>
                <span class="wf-entry-meta mt-4" style="text-transform:uppercase;letter-spacing:0.1em;">
                    {kicker}
                </span>
                <h1 class="wf-auth-title mt-4">{title}</h1>
                <p class="wf-auth-sub">{subtitle}</p>
                <div class="wf-entry-meta mt-12">
                    "── mithic · signal not noise ──"
                </div>
            </aside>
            <div class="wf-auth-form">
                <div class="wf-auth-inner">
                    {children()}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let toast = expect_context::<ToastStore>();
    let handle = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let remember = RwSignal::new(false);
    let show_pw = RwSignal::new(false);
    let error = RwSignal::<Option<String>>::new(None);
    let loading = RwSignal::new(false);
    let navigate = use_navigate();

    let on_submit = move |_| {
        let h = handle.get();
        let p = password.get();

        if h.trim().is_empty() {
            error.set(Some(
                "ユーザー名またはメールアドレスを入力してください".into(),
            ));
            return;
        }

        if p.len() < 8 {
            error.set(Some("パスワードは8文字以上で".into()));
            return;
        }
        error.set(None);
        loading.set(true);

        let auth2 = auth.clone();
        let nav2 = navigate.clone();
        let toast = toast;
        wasm_bindgen_futures::spawn_local(async move {
            use crate::api::auth::login;
            let req = crate::api::auth::LoginRequest {
                handle: h,
                password: p,
                remember: remember.get(),
            };
            match login(&req).await {
                Ok(pair) => {
                    auth2.login(pair.access_token, pair.user);
                    toast.push("ログインしました", ToastKind::Success);
                    nav2("/", Default::default());
                }
                Err(e) => {
                    error.set(Some(e.user_message()));
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <AuthShell
            kicker="[ LOG IN  01 ]"
            title="ようこそ、mithic。"
            subtitle="あなたの物語を、ここから続けましょう。"
        >
                    <span class="wf-mark wf-mark-md">"[m]"<span class="br">"mithic"</span></span>

                    <span class="wf-entry-meta">"[ 既存アカウント / SIGN IN ]"</span>
                    <h2 class="wf-auth-title" style="font-size:28px;">"ログイン"</h2>

                    <Show when=move || error.get().is_some()>
                        <div class="wf-alert error">
                            <span>{move || error.get().unwrap_or_default()}</span>
                        </div>
                    </Show>

                    <div class="flex flex-col gap-4 mt-4">
                        <label class="flex flex-col gap-1 w-full">
                            <span class="wf-entry-meta">"ハンドル / メール"</span>
                            <input class="wf-input"
                                placeholder="@hana"
                                prop:value=move || handle.get()
                                on:input=move |e| handle.set(event_target_value(&e))
                            />
                        </label>

                        <div>
                            <div class="flex justify-between items-center mb-1">
                                <span class="wf-entry-meta">"パスワード"</span>
                                <span class="text-xs cursor-pointer hover:underline" style="color:var(--accent);">"忘れた場合"</span>
                            </div>
                            <div class="wf-input flex items-center justify-between">
                                <input
                                    class="flex-1"
                                    style="background:transparent;border:none;outline:none;color:inherit;"
                                    prop:type=move || if show_pw.get() { "text" } else { "password" }
                                    placeholder="••••••••"
                                    prop:value=move || password.get()
                                    on:input=move |e| password.set(event_target_value(&e))
                                />
                                <button class="wf-btn wf-btn-ghost wf-btn-sm wf-btn-circle" on:click=move |_| show_pw.update(|v| *v = !*v)>
                                    {move || if show_pw.get() { "隠す" } else { "表示" }}
                                </button>
                            </div>
                        </div>

                        <div class="flex items-center justify-between text-xs mt-2">
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="checkbox" class="wf-check"
                                    prop:checked=move || remember.get()
                                    on:change=move |e| remember.set(event_target_checked(&e))
                                />
                                <span>"このブラウザを記憶"</span>
                            </label>
                        </div>

                        <button class="wf-btn wf-btn-primary mt-4" style="width:100%;"
                            disabled=move || loading.get()
                            on:click=on_submit>
                            {move || if loading.get() { "認証中…" } else { "ログイン →" }}
                        </button>

                        <p class="text-xs text-center opacity-60 mt-4">
                            "はじめての方は "
                            <A href="/signup" attr:class="font-bold" attr:style="color:var(--accent);">"新規登録 →"</A>
                        </p>
                    </div>
        </AuthShell>
    }
}

#[component]
pub fn SignupPage() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let navigate = use_navigate();
    let signup_handle = RwSignal::new(String::new());
    let display_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let password_confirm = RwSignal::new(String::new());
    let agreed_age = RwSignal::new(false);
    let agreed_tos = RwSignal::new(false);
    let handle_available = RwSignal::<Option<bool>>::new(None);
    let error = RwSignal::<Option<String>>::new(None);
    let busy = RwSignal::new(false);

    // 新規登録の実 API 呼び出し
    let do_register = move |_| {
        if busy.get_untracked() {
            return;
        }
        if handle_available.get_untracked() == Some(false) {
            error.set(Some("このハンドルは既に使用されています".into()));
            return;
        }
        busy.set(true);
        error.set(None);
        let auth = auth.clone();
        let navigate = navigate.clone();
        wasm_bindgen_futures::spawn_local(async move {
            use crate::api::auth::{RegisterRequest, register};
            let handle = signup_handle
                .get_untracked()
                .trim()
                .trim_start_matches('@')
                .to_string();
            let request = RegisterRequest {
                handle,
                display_name: Some(display_name.get_untracked()).filter(|s| !s.trim().is_empty()),
                email: Some(email.get_untracked()).filter(|s| !s.is_empty()),
                password: password.get_untracked(),
            };
            match register(&request).await {
                Ok(pair) => {
                    busy.set(false);
                    auth.login(pair.access_token, pair.user);
                    navigate("/", Default::default());
                }
                Err(e) => {
                    busy.set(false);
                    // network 等は client 側でユーザー向け文言にしているので message を優先
                    error.set(Some(e.message));
                }
            }
        });
    };

    // ハンドル可用性チェック (簡易デバウンス)
    // 入力は `@user` でも可。API 失敗時は None のまま（ローカル検証ではブロックしない）
    // 送信後にページを離れることがあるので、非リアクティブなフラグで破棄を検知する
    let disposed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    {
        let flag = disposed.clone();
        on_cleanup(move || flag.store(true, std::sync::atomic::Ordering::Relaxed));
    }
    Effect::new(move |_| {
        let raw = signup_handle.get();
        let h = raw.trim().trim_start_matches('@').to_string();
        if h.len() < 3 {
            handle_available.set(None);
            return;
        }
        let disposed = disposed.clone();
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;
            if disposed.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            let current = signup_handle
                .get_untracked()
                .trim()
                .trim_start_matches('@')
                .to_string();
            if current != h {
                return;
            }
            match crate::api::users::check_handle(&h).await {
                Ok(r) => handle_available.set(Some(r.available)),
                Err(_) => handle_available.set(None),
            }
        });
    });

    let pw_strength = Memo::new(move |_| {
        let p = password.get();
        let mut score = 0u8;
        if p.len() >= 8 {
            score += 1;
        }
        if p.len() >= 12 {
            score += 1;
        }
        if p.chars().any(|c| c.is_ascii_uppercase()) {
            score += 1;
        }
        if p.chars().any(|c| c.is_ascii_punctuation()) {
            score += 1;
        }
        score
    });

    // ボタン有効条件:
    // - ハンドル 3 文字以上、かつ API が「使用不可」と返していない（失敗時はブロックしない）
    // - 表示名・メール・パスワード一致・両方の同意
    let can_proceed = Memo::new(move |_| {
        let handle_ok = {
            let h = signup_handle.get();
            let normalized = h.trim().trim_start_matches('@');
            normalized.len() >= 3 && handle_available.get() != Some(false)
        };
        handle_ok
            && !display_name.get().trim().is_empty()
            && email.get().contains('@')
            && password.get().len() >= 8
            && password.get() == password_confirm.get()
            && agreed_age.get()
            && agreed_tos.get()
    });

    view! {
        <AuthShell
            kicker="[ SIGN UP  01 ]"
            title="アカウントを作成しましょう。"
            subtitle="mithic はオープンな分散型 SNS です。ActivityPub でつながります。"
        >
                    <div class="flex gap-1" style="height:4px;width:100%;border-radius:999px;overflow:hidden;background:var(--line-soft);">
                        <div style="background:var(--accent);flex:1;" />
                        <div style="background:var(--line-soft);flex:1;" />
                        <div style="background:var(--line-soft);flex:1;" />
                    </div>

                    <span class="wf-entry-meta mt-4">"[ STEP 1/3  登録情報 ]"</span>
                    <h2 class="wf-auth-title" style="font-size:28px;">"新規登録"</h2>

                    <Show when=move || error.get().is_some()>
                        <div class="wf-alert error">
                            <span>{move || error.get().unwrap_or_default()}</span>
                        </div>
                    </Show>

                    <div class="flex flex-col gap-4 mt-4">
                        <div>
                            <div class="flex justify-between items-center mb-1">
                                <span class="wf-entry-meta">"ハンドル"</span>
                                {move || match handle_available.get() {
                                    Some(true)  => view! { <span class="wf-pill on">"✓ 利用可能"</span> }.into_any(),
                                    Some(false) => view! { <span class="wf-pill" style="border-color:var(--err);color:var(--err);">"✕ 使用不可"</span> }.into_any(),
                                    None        => view! { <span></span> }.into_any(),
                                }}
                            </div>
                            <input class="wf-input"
                                placeholder="@hana"
                                prop:value=move || signup_handle.get()
                                on:input=move |e| signup_handle.set(event_target_value(&e))
                            />
                        </div>

                        <label class="flex flex-col gap-1 w-full">
                            <span class="wf-entry-meta">"表示名"</span>
                            <input class="wf-input"
                                placeholder="Hana K."
                                prop:value=move || display_name.get()
                                on:input=move |e| display_name.set(event_target_value(&e))
                            />
                        </label>

                        <label class="flex flex-col gap-1 w-full">
                            <span class="wf-entry-meta">"メールアドレス"</span>
                            <input class="wf-input"
                                type="email"
                                placeholder="hana@example.com"
                                prop:value=move || email.get()
                                on:input=move |e| email.set(event_target_value(&e))
                            />
                        </label>

                        <div>
                            <label class="flex flex-col gap-1 w-full">
                                <span class="wf-entry-meta">"パスワード"</span>
                                <input class="wf-input"
                                    type="password"
                                    placeholder="••••••••"
                                    prop:value=move || password.get()
                                    on:input=move |e| password.set(event_target_value(&e))
                                />
                            </label>
                            <div class="wf-pw-bar">
                                {move || (1..=4u8).map(|i| {
                                    let strength = pw_strength.get();
                                    let cls = if strength >= i {
                                        format!("wf-pw-seg s{i}")
                                    } else {
                                        "wf-pw-seg".into()
                                    };
                                    view! { <div class=cls /> }
                                }).collect_view()}
                            </div>
                        </div>

                        <label class="flex flex-col gap-1 w-full">
                            <span class="wf-entry-meta">"パスワード確認"</span>
                            <input class="wf-input"
                                type="password"
                                placeholder="••••••••"
                                prop:value=move || password_confirm.get()
                                on:input=move |e| password_confirm.set(event_target_value(&e))
                            />
                        </label>

                        <div class="flex flex-col gap-2 mt-2">
                            <label class="flex items-center gap-2 cursor-pointer text-xs">
                                <input type="checkbox" class="wf-check"
                                    prop:checked=move || agreed_age.get()
                                    on:change=move |e| agreed_age.set(event_target_checked(&e))
                                />
                                <span>"私は13歳以上です"</span>
                            </label>

                            <label class="flex items-center gap-2 cursor-pointer text-xs">
                                <input type="checkbox" class="wf-check"
                                    prop:checked=move || agreed_tos.get()
                                    on:change=move |e| agreed_tos.set(event_target_checked(&e))
                                />
                                <span>"利用規約に同意します"</span>
                            </label>
                        </div>

                        <button class="wf-btn wf-btn-primary mt-4" style="width:100%;"
                            disabled=move || !can_proceed.get() || busy.get()
                            on:click=do_register>
                            {move || if busy.get() { "登録中…" } else { "アカウント作成 →" }}
                        </button>

                        <Show when=move || !can_proceed.get() && !busy.get()>
                            <p class="text-xs text-center opacity-60 mt-2">
                                {move || {
                                    let h = signup_handle.get();
                                    let normalized = h.trim().trim_start_matches('@');
                                    if normalized.len() < 3 {
                                        "ハンドルは3文字以上で入力してください"
                                    } else if handle_available.get() == Some(false) {
                                        "このハンドルは使用できません"
                                    } else if display_name.get().trim().is_empty() {
                                        "表示名を入力してください"
                                    } else if !email.get().contains('@') {
                                        "有効なメールアドレスを入力してください"
                                    } else if password.get().len() < 8 {
                                        "パスワードは8文字以上で入力してください"
                                    } else if password.get() != password_confirm.get() {
                                        "パスワード確認が一致しません"
                                    } else if !agreed_age.get() {
                                        "年齢確認にチェックしてください"
                                    } else if !agreed_tos.get() {
                                        "利用規約への同意が必要です"
                                    } else {
                                        ""
                                    }
                                }}
                            </p>
                        </Show>

                        <p class="text-xs text-center opacity-60 mt-4">
                            "既にアカウントをお持ちの方は "
                            <A href="/login" attr:class="font-bold" attr:style="color:var(--accent);">"ログイン →"</A>
                        </p>
                    </div>
        </AuthShell>
    }
}

#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <Shell active="settings">
            <TopBar title="管理コンソール" />
            <section class="wf-scroll p-6 flex flex-col items-center">
                <div class="wf-card max-w-md w-full text-center flex flex-col gap-3">
                    <span class="wf-entry-meta">"ADMIN"</span>
                    <h1 class="wf-title">"管理機能は準備中です"</h1>
                    <p class="text-sm opacity-70">
                        "ユーザー管理・モデレーション・インスタンス統計は今後のリリースで追加します。"
                    </p>
                    <A href="/" attr:class="wf-btn wf-btn-primary" attr:style="width:100%;">
                        "ホームに戻る"
                    </A>
                </div>
            </section>
        </Shell>
    }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <Shell active="home">
            <section class="p-4 flex flex-col items-center justify-center min-h-[50dvh]">
                <div class="wf-card max-w-sm text-center flex flex-col items-center gap-4">
                    <span class="wf-entry-meta">"[ 404 ]"</span>
                    <h1 class="wf-title">"アカウントが見つかりません"</h1>
                    <A href="/" attr:class="wf-btn wf-btn-primary" attr:style="width:100%;">"ホームに戻る"</A>
                </div>
            </section>
        </Shell>
    }
}
