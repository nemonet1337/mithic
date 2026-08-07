use gloo_storage::{LocalStorage, Storage};
use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};

use crate::components::{
    Avatar, AvatarSize, FollowButton, LoadMore, MarkdownText, PostCard, Shell, ToastKind,
    ToastStore, TopBar,
};
use crate::models::{Note, NotificationType};
use crate::store::{AuthStore, NotificationStore, stream::connect_stream};

mod drive;
pub use drive::DrivePage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimelineKind {
    Home,
    Local,
    Global,
}

#[component]
pub fn HomePage() -> impl IntoView {
    view! { <TimelinePage kind=TimelineKind::Home /> }
}

#[component]
pub fn LocalTimelinePage() -> impl IntoView {
    view! { <TimelinePage kind=TimelineKind::Local /> }
}

#[component]
pub fn GlobalTimelinePage() -> impl IntoView {
    view! { <TimelinePage kind=TimelineKind::Global /> }
}

#[component]
fn TimelinePage(kind: TimelineKind) -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let notifications = expect_context::<NotificationStore>();
    let notes = RwSignal::<Vec<Note>>::new(vec![]);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);

    let kind_str = match kind {
        TimelineKind::Home => "home",
        TimelineKind::Local => "local",
        TimelineKind::Global => "global",
    };
    let active_path = match kind {
        TimelineKind::Home => "/",
        TimelineKind::Local => "/local",
        TimelineKind::Global => "/global",
    };

    let tabs = vec![
        (id::FiHome, "ホーム", "/", active_path == "/"),
        (id::FiUsers, "ローカル", "/local", active_path == "/local"),
        (id::FiGlobe, "グローバル", "/global", active_path == "/global"),
    ];

    // タイムライン読み込み
    Effect::new(move |_| {
        let token = auth.token.get();
        if let Some(tok) = token {
            is_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::notes::fetch_timeline(&tok, kind_str, None).await {
                    Ok(fetched) => {
                        notes.set(fetched);
                        is_loading.set(false);
                    }
                    Err(e) => {
                        web_sys::console::error_1(&e.to_string().into());
                        is_loading.set(false);
                    }
                }
            });
        }
    });

    // WebSocket でリアルタイム先頭へ挿入
    Effect::new(move |_| {
        if let Some(token) = auth.token.get() {
            connect_stream(token, notes, notifications);
        }
    });

    let load_more = move || {
        let token = auth.token.get_untracked();
        let oldest = notes.with_untracked(|v| v.last().map(|n| n.id.clone()));
        if let (Some(tok), Some(id)) = (token, oldest) {
            is_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::notes::fetch_timeline(&tok, kind_str, Some(&id)).await {
                    Ok(mut more) => {
                        if more.is_empty() {
                            has_more.set(false);
                        }
                        notes.update(|v| v.append(&mut more));
                        is_loading.set(false);
                    }
                    Err(e) => {
                        web_sys::console::error_1(&e.to_string().into());
                        is_loading.set(false);
                    }
                }
            });
        }
    };

    view! {
        <Shell active="home">
            <TopBar tabs=tabs />
            <section class="wf-scroll">
                <For
                    each=move || notes.get()
                    key=|note| note.id.clone()
                    children=|note| view! { <PostCard note=note /> }
                />
                <Show when=move || is_loading.get()>
                    <div class="flex items-center justify-center gap-2 py-4">
                        <span class="wf-spinner" style="width:18px;height:18px;" />
                        <span class="wf-entry-meta">"読み込み中…"</span>
                    </div>
                </Show>
                <Show when=move || !is_loading.get() && has_more.get() && !notes.get().is_empty()>
                    <LoadMore on_visible=std::sync::Arc::new(move || load_more()) />
                </Show>
            </section>
        </Shell>
    }
}

#[component]
pub fn StatusDetailPage() -> impl IntoView {
    let params = use_params_map();
    let auth = expect_context::<AuthStore>();
    let note = RwSignal::<Option<Note>>::new(None);
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

    view! {
        <Shell active="home">
            <TopBar title="投稿詳細" />
            <div class="flex flex-col gap-4 p-4 wf-scroll">
                <Show when=move || loading.get()>
                    <div class="flex items-center justify-center gap-2 py-8">
                        <span class="wf-spinner" style="width:18px;height:18px;" />
                        <span class="wf-entry-meta">"読み込み中…"</span>
                    </div>
                </Show>
                <Show when=move || error.get().is_some()>
                    <div class="wf-alert error">
                        <span>{move || error.get().unwrap_or_default()}</span>
                    </div>
                </Show>
                {move || note.get().map(|current| {
                    let reaction_row = if current.reactions.is_empty() {
                        ().into_any()
                    } else {
                        view! {
                            <section class="wf-card">
                                <span class="wf-entry-meta">"リアクション"</span>
                                <div class="flex flex-wrap gap-2 mt-2">
                                    {current
                                        .reactions
                                        .iter()
                                        .map(|r| {
                                            let label = format!("{} {}", r.emoji, r.count);
                                            view! { <span class="wf-pill">{label}</span> }
                                        })
                                        .collect_view()}
                                </div>
                            </section>
                        }
                        .into_any()
                    };
                    view! {
                        <section class="flex flex-col gap-4">
                            <PostCard note=current flat=true />
                            {reaction_row}
                            <span class="wf-entry-meta">"返信"</span>
                            <Show when=move || replies.get().is_empty()>
                                <div class="wf-dashed p-6 text-center">
                                    <span class="wf-entry-meta">"まだ返信はありません"</span>
                                </div>
                            </Show>
                            <For
                                each=move || replies.get()
                                key=|n| n.id.clone()
                                children=|n| view! { <PostCard note=n /> }
                            />
                        </section>
                    }
                })}
            </div>
        </Shell>
    }
}

#[component]
pub fn NotificationsPage() -> impl IntoView {
    let notification_store = expect_context::<NotificationStore>();
    let auth = expect_context::<AuthStore>();
    let token = auth.token;
    let notifications = RwSignal::<Vec<crate::models::Notification>>::new(vec![]);
    let filter = RwSignal::new("all");

    // 実 API から通知一覧を取得
    Effect::new(move |_| {
        if let Some(tok) = token.get() {
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::notifications::fetch_notifications(&tok, None).await {
                    Ok(fetched) => notifications.set(fetched),
                    Err(e) => web_sys::console::error_1(&e.to_string().into()),
                }
            });
        }
    });

    let filtered_notifications = move || {
        let items = notifications.get();
        let f = filter.get();
        items
            .into_iter()
            .filter(|n| match f {
                "mention" => n.notification_type == NotificationType::Reply,
                "reaction" => n.notification_type == NotificationType::Reaction,
                "follow" => n.notification_type == NotificationType::Follow,
                _ => true,
            })
            .collect::<Vec<_>>()
    };

    let mark_all_read = move |_| {
        notification_store.mark_notifications_read();
        notifications.update(|items| items.iter_mut().for_each(|n| n.is_read = true));
        if let Some(tok) = token.get_untracked() {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = crate::api::notifications::mark_all_read(&tok).await {
                    web_sys::console::error_1(&e.to_string().into());
                }
            });
        }
    };
    view! {
        <Shell active="notif">
            <TopBar title="通知" />
            <div class="flex items-center justify-between px-4 py-2">
                <Show when=move || { notification_store.unread_notifications.get() > 0 }>
                    <span class="wf-pill on">
                        "未読 " {move || notification_store.unread_notifications.get().to_string()}
                    </span>
                </Show>
                <button
                    class="wf-btn wf-btn-ghost wf-btn-sm wf-btn-circle ml-auto"
                    on:click=mark_all_read
                    aria-label="すべて既読にする"
                    title="すべて既読にする"
                >
                    <Icon icon=id::FiCheckCircle width="18" height="18" />
                </button>
            </div>
            <div class="wf-seg px-4">
                <span
                    class=move || if filter.get() == "all" { "wf-seg-item active" } else { "wf-seg-item" }
                    on:click=move |_| filter.set("all")>
                    "すべて"
                </span>
                <span
                    class=move || if filter.get() == "mention" { "wf-seg-item active" } else { "wf-seg-item" }
                    on:click=move |_| filter.set("mention")>
                    "返信"
                </span>
                <span
                    class=move || if filter.get() == "reaction" { "wf-seg-item active" } else { "wf-seg-item" }
                    on:click=move |_| filter.set("reaction")>
                    "リアクション"
                </span>
                <span
                    class=move || if filter.get() == "follow" { "wf-seg-item active" } else { "wf-seg-item" }
                    on:click=move |_| filter.set("follow")>
                    "フォロー"
                </span>
            </div>
            <section class="wf-scroll">
                <For
                    each=filtered_notifications
                    key=|notification| notification.id.clone()
                    children=|notification| {
                    let sender = notification.sender.clone();
                    let note   = notification.note.clone();
                    let unread_class = if notification.is_read { "wf-notif" } else { "wf-notif unread" };
                    let kind_label = match notification.notification_type {
                        NotificationType::Reaction => format!(
                            "{} があなたの投稿にリアクションしました",
                            notification.reaction.as_deref().unwrap_or("誰か")
                        ),
                        NotificationType::Reply => "があなたの投稿に返信しました".into(),
                        NotificationType::Follow => "があなたをフォローしました".into(),
                        NotificationType::Renote => "があなたの投稿をリノートしました".into(),
                        NotificationType::Mention => "があなたをメンションしました".into(),
                        NotificationType::Quote => "があなたの投稿を引用しました".into(),
                        NotificationType::FollowRequest => "がフォローリクエストを送信しました".into(),
                        NotificationType::FollowRequestAccepted => {
                            "があなたのフォローリクエストを承認しました".into()
                        }
                        NotificationType::PollEnded => "のアンケートが終了しました".into(),
                        NotificationType::UserSignup => "が登録しました".into(),
                    };
                    view! {
                        <article class=unread_class>
                            {sender.map(|user| view! { <Avatar user=user size=AvatarSize::Sm /> }).into_view()}
                            <div class="wf-notif-text">
                                <div class="flex items-center justify-between">
                                    <span class="who">{kind_label}</span>
                                    <span class="wf-notif-time">{notification.created_at}</span>
                                </div>
                                {note.map(|n| view! {
                                    <blockquote class="wf-dashed mt-2 p-3 text-sm"><MarkdownText text=n.content /></blockquote>
                                }).into_view()}
                            </div>
                        </article>
                    }
                } />
            </section>
        </Shell>
    }
}

#[component]
pub fn SearchPage() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let query = use_query_map();
    let navigate = use_navigate();

    let search_input = RwSignal::new(query.read().get("q").unwrap_or_default());
    let notes = RwSignal::<Vec<Note>>::new(Vec::new());
    let users = RwSignal::<Vec<crate::models::User>>::new(Vec::new());
    let trend_tags = RwSignal::<Vec<shared::Hashtag>>::new(Vec::new());
    let loading = RwSignal::new(false);
    let searched = RwSignal::new(false);

    Effect::new(move |_| {
        let q = query.read().get("q").unwrap_or_default();
        search_input.set(q);
    });

    // トレンドタグ（ピル用）
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(tags) = crate::api::notes::fetch_trending(6).await {
                trend_tags.set(tags);
            }
        });
    });

    // クエリ変更で API 検索
    Effect::new(move |_| {
        let q_val = query.read().get("q").unwrap_or_default();
        let tag_val = query.read().get("tag").unwrap_or_default();
        let tok = auth.token.get();

        if q_val.trim().is_empty() && tag_val.trim().is_empty() {
            notes.set(Vec::new());
            users.set(Vec::new());
            searched.set(false);
            return;
        }

        loading.set(true);
        searched.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            if !tag_val.is_empty() {
                match crate::api::notes::fetch_hashtag_timeline(tok.as_deref(), &tag_val, 30).await
                {
                    Ok(list) => {
                        notes.set(list);
                        users.set(Vec::new());
                    }
                    Err(e) => {
                        web_sys::console::error_1(&e.to_string().into());
                        notes.set(Vec::new());
                    }
                }
            } else {
                let q = q_val.clone();
                match crate::api::notes::search_notes(tok.as_deref(), &q, 30).await {
                    Ok(list) => notes.set(list),
                    Err(e) => {
                        web_sys::console::error_1(&e.to_string().into());
                        notes.set(Vec::new());
                    }
                }
                match crate::api::users::search_users(tok.as_deref(), &q).await {
                    Ok(list) => users.set(list),
                    Err(e) => {
                        web_sys::console::error_1(&e.to_string().into());
                        users.set(Vec::new());
                    }
                }
            }
            loading.set(false);
        });
    });

    let nav_enter = navigate.clone();
    let nav_click = navigate;

    view! {
        <Shell active="search">
            <TopBar title="検索 / 発見" />
            <section class="wf-scroll p-4 flex flex-col gap-4">
                <div class="wf-card flex flex-col gap-3">
                    <span class="wf-entry-meta">"検索"</span>
                    <div class="flex gap-2 w-full">
                        <input
                            class="wf-input flex-1"
                            placeholder="投稿・ユーザー・タグを検索"
                            prop:value=move || search_input.get()
                            on:input=move |ev| search_input.set(event_target_value(&ev))
                            on:keydown=move |ev| {
                                if ev.key() == "Enter" {
                                    let q = search_input.get();
                                    nav_enter(&format!("/search?q={}", q), Default::default());
                                }
                            }
                        />
                        <button
                            class="wf-btn wf-btn-primary"
                            on:click=move |_| {
                                let q = search_input.get();
                                nav_click(&format!("/search?q={}", q), Default::default());
                            }
                        >
                            "検索"
                        </button>
                    </div>
                    <Show when=move || !trend_tags.get().is_empty()>
                        <div class="flex flex-wrap gap-2 mt-2">
                            {move || trend_tags.get().into_iter().map(|h| {
                                let tag = h.tag.clone();
                                let bare = tag.trim_start_matches('#').to_string();
                                view! {
                                    <A href=format!("/search?tag={}", bare) attr:class="wf-pill">{tag}</A>
                                }
                            }).collect_view()}
                        </div>
                    </Show>
                </div>

                <Show when=move || loading.get()>
                    <div class="flex items-center justify-center gap-2 py-6">
                        <span class="wf-spinner" style="width:18px;height:18px;" />
                        <span class="wf-entry-meta">"検索中…"</span>
                    </div>
                </Show>

                <Show when=move || !loading.get() && !users.get().is_empty()>
                    <div class="wf-card flex flex-col gap-2">
                        <span class="wf-entry-meta">"ユーザー"</span>
                        {move || users.get().into_iter().map(|u| {
                            let href = format!("/profile/{}", u.route_handle());
                            let name = u.name();
                            let handle = u.handle();
                            view! {
                                <A href=href attr:class="flex items-center gap-3 p-2 hover:underline">
                                    <Avatar user=u size=AvatarSize::Sm />
                                    <div class="min-w-0">
                                        <div class="font-bold text-sm truncate">{name}</div>
                                        <div class="wf-entry-meta truncate">{handle}</div>
                                    </div>
                                </A>
                            }
                        }).collect_view()}
                    </div>
                </Show>

                <div class="flex flex-col gap-3">
                    {move || {
                        if loading.get() {
                            return ().into_any();
                        }
                        let list = notes.get();
                        if !searched.get() {
                            view! {
                                <div class="wf-dashed p-8 text-center">
                                    <span class="wf-entry-meta">"キーワードまたはタグで検索できます。"</span>
                                </div>
                            }.into_any()
                        } else if list.is_empty() && users.get().is_empty() {
                            view! {
                                <div class="wf-dashed p-8 text-center">
                                    <span class="wf-entry-meta">"検索結果が見つかりませんでした。"</span>
                                </div>
                            }.into_any()
                        } else {
                            list.into_iter().map(|note| view! { <PostCard note=note /> }).collect_view().into_any()
                        }
                    }}
                </div>
            </section>
        </Shell>
    }
}

#[component]
pub fn DmPage() -> impl IntoView {
    view! { <DmScaffold conversation_id=None /> }
}

#[component]
pub fn DmConversationPage() -> impl IntoView {
    let params = use_params_map();
    let conversation_id = params.read().get("conversation");
    view! { <DmScaffold conversation_id=conversation_id /> }
}

/// DM は API 未接続。空状態のみ（ルートは維持）。
#[component]
fn DmScaffold(conversation_id: Option<String>) -> impl IntoView {
    let notifications = expect_context::<NotificationStore>();
    Effect::new(move |_| notifications.mark_messages_read());
    let _ = conversation_id;

    view! {
        <Shell active="dm">
            <div class="wf-dm">
                <aside class="wf-dm-list flex flex-col">
                    <div class="flex items-center justify-between p-4" style="border-bottom:1px solid var(--line-soft);">
                        <span class="wf-title">"メッセージ"</span>
                    </div>
                    <div class="flex-1 overflow-y-auto flex flex-col items-center justify-center p-6">
                        <div class="wf-empty" style="padding:24px 8px;">
                            <span>"ダイレクトメッセージは準備中です"</span>
                        </div>
                    </div>
                </aside>
                <main class="wf-dm-conv">
                    <div class="wf-dm-msgs flex-1 flex flex-col items-center justify-center p-8">
                        <div class="wf-empty">
                            <span>"会話 API の接続後にここに表示されます"</span>
                        </div>
                    </div>
                </main>
            </div>
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
    let user = RwSignal::<Option<crate::models::User>>::new(None);
    let notes = RwSignal::<Vec<Note>>::new(vec![]);
    let is_following = RwSignal::new(false);
    let follow_busy = RwSignal::new(false);
    let profile_tab = RwSignal::new("notes");

    // プロフィールと投稿一覧を実 API から取得
    Effect::new(move |_| {
        let username = handle();
        if let Some(tok) = token.get() {
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::users::fetch_user(&tok, &username).await {
                    Ok(fetched) => user.set(Some(fetched)),
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
        let currently = is_following.get_untracked();
        let toast = toast;
        wasm_bindgen_futures::spawn_local(async move {
            let result = if currently {
                crate::api::users::unfollow(&tok, &target.id).await
            } else {
                crate::api::users::follow(&tok, &target.id).await
            };
            follow_busy.set(false);
            match result {
                Ok(()) => {
                    is_following.set(!currently);
                    toast.push(
                        if currently {
                            "フォローを解除しました"
                        } else {
                            "フォローしました"
                        },
                        ToastKind::Success,
                    );
                }
                Err(e) => toast.push(e.user_message(), ToastKind::Error),
            }
        });
    });

    view! {
        <Shell active="profile">
            <section class="wf-scroll">
                <div class="relative">
                    <div class="wf-profile-banner" />
                    <div class="wf-profile-head">
                        <div class="wf-profile-av">
                            {move || user.get().map(|u| view! { <Avatar user=u size=AvatarSize::Xl /> })}
                        </div>
                        <div class="wf-profile-meta">
                            <div class="flex items-center justify-between">
                                <div>
                                    <span class="wf-entry-meta">"ACTIVITYPUB  " {move || if handle().contains('@') { "REMOTE" } else { "LOCAL" }}</span>
                                    <h1 class="wf-profile-name mt-1">{move || user.get().map(|u| u.name()).unwrap_or_default()}</h1>
                                    <span class="wf-profile-handle">{move || format!("@{}", handle())}</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <Show when=move || auth.me.get().zip(user.get()).map(|(me, u)| me.id != u.id).unwrap_or(false)>
                                        <FollowButton
                                            is_following=is_following
                                            is_pending=follow_busy
                                            on_toggle=toggle_follow
                                        />
                                    </Show>
                                </div>
                            </div>
                            <p class="text-sm mt-2">{move || user.get().and_then(|u| u.bio).unwrap_or_default()}</p>

                            <div class="wf-profile-stats">
                                <div class="wf-stat">
                                    <span class="v">{move || user.get().map(|u| u.notes_count).unwrap_or(0).to_string()}</span>
                                    <span class="l">"投稿"</span>
                                </div>
                                <div class="wf-stat">
                                    <span class="v">{move || user.get().map(|u| u.followers_count).unwrap_or(0).to_string()}</span>
                                    <span class="l">"フォロワー"</span>
                                </div>
                                <div class="wf-stat">
                                    <span class="v">{move || user.get().map(|u| u.following_count).unwrap_or(0).to_string()}</span>
                                    <span class="l">"フォロー"</span>
                                </div>
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
                                        each=move || notes.get()
                                        key=|note| note.id.clone()
                                        children=|note| view! { <PostCard note=note /> }
                                    />
                                }.into_any()
                            }
                        }
                    }}
                </div>
            </section>
        </Shell>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let toast = expect_context::<ToastStore>();
    let token = auth.token;
    let me = auth.me;

    let params = use_params_map();
    let section = move || -> String {
        let raw = params
            .read()
            .get("section")
            .unwrap_or_else(|| "プロフィール".into());
        // 実装済みセクション以外はプロフィールへ寄せる（古い URL 互換）
        match raw.as_str() {
            "テーマ" => "テーマ".to_string(),
            _ => "プロフィール".to_string(),
        }
    };

    let display_name_signal = RwSignal::new(String::new());
    let bio_signal = RwSignal::new(String::new());
    let handle_signal = RwSignal::new(String::new());
    let save_busy = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(user) = me.get() {
            display_name_signal.set(user.display_name.clone().unwrap_or_default());
            bio_signal.set(user.bio.clone().unwrap_or_default());
            handle_signal.set(user.handle());
        }
    });

    let on_reset = move |_| {
        if let Some(user) = me.get_untracked() {
            display_name_signal.set(user.display_name.clone().unwrap_or_default());
            bio_signal.set(user.bio.clone().unwrap_or_default());
        }
    };

    let on_save = move |_| {
        if save_busy.get_untracked() {
            return;
        }
        let Some(tok) = token.get_untracked() else {
            return;
        };
        let req = crate::api::users::UpdateProfileRequest {
            display_name: Some(display_name_signal.get_untracked()),
            bio: Some(bio_signal.get_untracked()),
        };
        save_busy.set(true);
        let toast = toast;
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::users::update_me(&tok, &req).await {
                Ok(updated_user) => {
                    auth.me.set(Some(updated_user));
                    toast.push("プロフィールを保存しました", ToastKind::Success);
                }
                Err(e) => toast.push(e.user_message(), ToastKind::Error),
            }
            save_busy.set(false);
        });
    };

    // 実装済み: プロフィール / テーマのみ
    let nav_items = [("プロフィール", "アカウント"), ("テーマ", "表示")];

    let theme = RwSignal::new(
        LocalStorage::get::<String>("mithic.theme").unwrap_or_else(|_| "night".into()),
    );

    let set_theme = move |t: &'static str| {
        let is_dark = match t {
            "light" => false,
            "auto" => web_sys::window()
                .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
                .map(|mql| mql.matches())
                .unwrap_or(true),
            // "dark" | "night" | default
            _ => true,
        };
        theme.set(t.into());
        let _ = LocalStorage::set("mithic.theme", t);
        if let Some(html) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            let class = html.class_name();
            let next = if is_dark {
                if class.split_whitespace().any(|c| c == "dark") {
                    class
                } else if class.is_empty() {
                    "dark".into()
                } else {
                    format!("{class} dark")
                }
            } else {
                class
                    .split_whitespace()
                    .filter(|c| *c != "dark")
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            let _ = html.set_class_name(&next);
            let _ = html.set_attribute("data-theme", if is_dark { "night" } else { "light" });
        }
    };

    view! {
        <Shell active="settings">
            <div class="wf-settings-layout flex" style="height:100%;overflow:hidden;">
                <aside class="wf-rail" style="width:220px;flex-shrink:0;">
                    <span class="wf-title" style="font-size:18px;">"設定"</span>
                    {nav_items.into_iter().map(|(item, group)| {
                        let is_active = move || section() == item;
                        view! {
                            <div>
                                <span class="wf-rail-tag" style="display:block;margin:8px 0 4px;">{group}</span>
                                <A href=format!("/settings/{item}")
                                    attr:class=move || if is_active() { "wf-pop-item active" } else { "wf-pop-item" }>
                                    {item}
                                </A>
                            </div>
                        }
                    }).collect_view()}
                </aside>
                <main class="wf-scroll p-6" style="flex:1;">
                    {move || match section().as_str() {
                        "テーマ" => view! {
                            <span class="wf-entry-meta">"表示 / テーマ"</span>
                            <h1 class="wf-title mt-1 mb-6">"テーマ設定"</h1>
                            <div class="wf-card max-w-md flex flex-row items-center justify-between">
                                <span class="text-sm font-semibold">"テーマ"</span>
                                <div class="flex gap-1">
                                    <button
                                        class=move || if theme.get() == "light" { "wf-btn wf-btn-primary wf-btn-sm" } else { "wf-btn wf-btn-ghost wf-btn-sm" }
                                        on:click=move |_| set_theme("light")>
                                        "ライト"
                                    </button>
                                    <button
                                        class=move || if theme.get() == "dark" || theme.get() == "night" { "wf-btn wf-btn-primary wf-btn-sm" } else { "wf-btn wf-btn-ghost wf-btn-sm" }
                                        on:click=move |_| set_theme("night")>
                                        "ダーク"
                                    </button>
                                    <button
                                        class=move || if theme.get() == "auto" { "wf-btn wf-btn-primary wf-btn-sm" } else { "wf-btn wf-btn-ghost wf-btn-sm" }
                                        on:click=move |_| set_theme("auto")>
                                        "自動"
                                    </button>
                                </div>
                            </div>
                        }.into_any(),
                        _ => view! {
                            <span class="wf-entry-meta">"アカウント / プロフィール"</span>
                            <h1 class="wf-title mt-1 mb-6">"プロフィール設定"</h1>
                            <div class="flex flex-col gap-4 max-w-md">
                                {move || me.get().map(|u| view! {
                                    <Avatar user=u size=AvatarSize::Xl />
                                })}
                                <label class="flex flex-col gap-1 w-full">
                                    <span class="wf-entry-meta">"表示名"</span>
                                    <input
                                        class="wf-input"
                                        prop:value=move || display_name_signal.get()
                                        on:input=move |ev| display_name_signal.set(event_target_value(&ev))
                                    />
                                </label>
                                <label class="flex flex-col gap-1 w-full">
                                    <span class="wf-entry-meta">"ハンドル"</span>
                                    <input
                                        class="wf-input"
                                        prop:value=move || handle_signal.get()
                                        disabled=true
                                    />
                                </label>
                                <label class="flex flex-col gap-1 w-full">
                                    <span class="wf-entry-meta">"自己紹介"</span>
                                    <textarea
                                        class="wf-input"
                                        style="height:96px;resize:none;"
                                        prop:value=move || bio_signal.get()
                                        on:input=move |ev| bio_signal.set(event_target_value(&ev))
                                    />
                                </label>
                                <div class="flex items-center justify-end gap-2 mt-4">
                                    <button class="wf-btn wf-btn-ghost" on:click=on_reset>"リセット"</button>
                                    <button
                                        class="wf-btn wf-btn-primary"
                                        disabled=move || save_busy.get()
                                        on:click=on_save
                                    >
                                        {move || if save_busy.get() { "保存中…" } else { "保存" }}
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }}
                </main>
            </div>
        </Shell>
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
    Effect::new(move |_| {
        let raw = signup_handle.get();
        let h = raw.trim().trim_start_matches('@').to_string();
        if h.len() < 3 {
            handle_available.set(None);
            return;
        }
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;
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
