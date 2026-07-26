use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};

use crate::components::{Avatar, AvatarSize, LoadMore, MarkdownText, PostCard, Shell, TopBar};
use crate::models::{Note, NotificationType, sample_notes};
use crate::store::{AuthStore, NotificationStore, stream::connect_stream};

mod drive;
pub use drive::DrivePage;

const TIMELINE_TABS: [(&str, &str); 3] = [
    ("ホーム", "/"),
    ("ローカル", "/local"),
    ("グローバル", "/global"),
];

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

    let tabs: Vec<(&'static str, &'static str, bool)> = TIMELINE_TABS
        .iter()
        .map(|(label, href)| (*label, *href, *href == active_path))
        .collect();

    let title = match kind {
        TimelineKind::Home => "ホーム",
        TimelineKind::Local => "ローカル",
        TimelineKind::Global => "グローバル",
    };

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
            <TopBar title=title folio="01" tabs=tabs />
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
    let notes = sample_notes();
    let current = notes
        .first()
        .cloned()
        .unwrap_or_else(|| sample_notes().remove(0));
    let reply_placeholder = format!("{} への返信", current.author.handle());
    view! {
        <Shell active="home">
            <TopBar title="投稿詳細" folio="02" />
            <div class="flex flex-col lg:flex-row gap-4 p-4 wf-scroll">
                <section class="flex-1 flex flex-col gap-4">
                    <PostCard note=current.clone() flat=true />
                    <div class="wf-card flex flex-col gap-3">
                        <span class="wf-entry-meta">"返信先: " {current.author.handle()}</span>
                        <textarea class="wf-compose-area" style="min-height:64px;" placeholder=reply_placeholder />
                        <div class="flex items-center justify-between">
                            <div class="flex items-center gap-2">
                                <button class="wf-btn wf-btn-ghost wf-btn-sm">"添付"</button>
                                <button class="wf-btn wf-btn-ghost wf-btn-sm">"絵文字"</button>
                            </div>
                            <button class="wf-btn wf-btn-primary wf-btn-sm">"返信"</button>
                        </div>
                    </div>
                    {notes.into_iter().skip(1).map(|note| view! { <PostCard note=note /> }).collect_view()}
                </section>
                <aside class="w-full lg:w-80 flex flex-col gap-4">
                    <section class="wf-card">
                        <span class="wf-entry-meta">"[ REACTIONS ]"</span>
                        <div class="flex flex-wrap gap-2 mt-2">
                            {current.reactions.iter().map(|reaction| view! {
                                <span class="wf-pill">{reaction.emoji.clone()} " " {reaction.count.to_string()}</span>
                            }).collect_view()}
                        </div>
                    </section>
                    <section class="wf-card">
                        <span class="wf-entry-meta">"[ QUOTES ]"</span>
                        <p class="text-sm mt-2">"この投稿を引用した投稿がここに表示されます。"</p>
                    </section>
                </aside>
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
            <TopBar title="通知" folio="03" />
            <div class="flex items-center justify-between px-4 py-2">
                <Show when=move || (notification_store.unread_notifications.get() > 0)>
                    <span class="wf-pill on">
                        "未読 " {move || notification_store.unread_notifications.get().to_string()}
                    </span>
                </Show>
                <button class="wf-btn wf-btn-ghost wf-btn-sm ml-auto"
                    on:click=mark_all_read>
                    "すべて既読に"
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
                    let (kind_label, action_view) = match notification.notification_type {
                        NotificationType::Reaction => (
                            format!("{} があなたの投稿にリアクションしました", notification.reaction.as_deref().unwrap_or("誰か")),
                            view! {}.into_any(),
                        ),
                        NotificationType::Reply => (
                            "があなたの投稿に返信しました".into(),
                            view! {
                                <div class="flex items-center gap-2 mt-3">
                                    <button class="wf-btn wf-btn-ghost wf-btn-sm">"返信"</button>
                                    <button class="wf-btn wf-btn-sm">"開く"</button>
                                </div>
                            }.into_any(),
                        ),
                        NotificationType::Follow => (
                            "があなたをフォローしました".into(),
                            view! {
                                <div class="mt-3">
                                    <button class="wf-btn wf-btn-primary wf-btn-sm">"フォローバック"</button>
                                </div>
                            }.into_any(),
                        ),
                        NotificationType::Renote => (
                            "があなたの投稿をリノートしました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::Mention => (
                            "があなたをメンションしました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::Quote => (
                            "があなたの投稿を引用しました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::FollowRequest => (
                            "がフォローリクエストを送信しました".into(),
                            view! {
                                <div class="flex items-center gap-2 mt-3">
                                    <button class="wf-btn wf-btn-primary wf-btn-sm">"承認"</button>
                                    <button class="wf-btn wf-btn-ghost wf-btn-sm">"拒否"</button>
                                </div>
                            }.into_any(),
                        ),
                        NotificationType::FollowRequestAccepted => (
                            "があなたのフォローリクエストを承認しました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::PollEnded => (
                            "のアンケートが終了しました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::UserSignup => (
                            "が登録しました".into(),
                            view! {}.into_any(),
                        ),
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
                                {action_view}
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
    let query = use_query_map();
    let navigate = use_navigate();

    // query の "q" 値で初期化
    let search_input = RwSignal::new(query.read().get("q").unwrap_or_default());

    // URL クエリパラメータの変更を監視して入力欄に反映
    Effect::new(move |_| {
        let q = query.read().get("q").unwrap_or_default();
        search_input.set(q);
    });

    let navigate_click = navigate.clone();
    let do_search_click = move || {
        let q = search_input.get();
        navigate_click(&format!("/search?q={}", q), Default::default());
    };

    let navigate_key = navigate.clone();
    let do_search_key = move || {
        let q = search_input.get();
        navigate_key(&format!("/search?q={}", q), Default::default());
    };

    let filtered_notes = move || {
        let q_val = query.read().get("q").unwrap_or_default();
        let tag_val = query.read().get("tag").unwrap_or_default();
        let all_notes = sample_notes();

        all_notes
            .into_iter()
            .filter(|note| {
                if !q_val.is_empty() {
                    note.content.to_lowercase().contains(&q_val.to_lowercase())
                        || note
                            .author
                            .name()
                            .to_lowercase()
                            .contains(&q_val.to_lowercase())
                        || note
                            .author
                            .handle()
                            .to_lowercase()
                            .contains(&q_val.to_lowercase())
                } else if !tag_val.is_empty() {
                    note.tags
                        .iter()
                        .any(|t| t.to_lowercase() == tag_val.to_lowercase())
                        || note
                            .content
                            .to_lowercase()
                            .contains(&format!("#{}", tag_val.to_lowercase()))
                } else {
                    true
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <Shell active="search">
            <TopBar title="検索 / 発見" folio="04" />
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
                                    do_search_key();
                                }
                            }
                        />
                        <button class="wf-btn wf-btn-primary" on:click=move |_| do_search_click()>"検索"</button>
                    </div>
                    <div class="flex flex-wrap gap-2 mt-2">
                        {vec!["#art", "#tech", "#books", "#music", "#food", "#photo"].into_iter().map(|tag| view! {
                            <A href=format!("/search?tag={}", tag.trim_start_matches('#')) attr:class="wf-pill">{tag}</A>
                        }).collect_view()}
                    </div>
                </div>
                <div class="flex flex-col gap-3">
                    {move || {
                        let notes = filtered_notes();
                        if notes.is_empty() {
                            view! {
                                <div class="wf-dashed p-8 text-center">
                                    <span class="wf-entry-meta">"検索結果が見つかりませんでした。"</span>
                                </div>
                            }.into_any()
                        } else {
                            notes.into_iter().map(|note| view! { <PostCard note=note /> }).collect_view().into_any()
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

#[component]
fn DmScaffold(conversation_id: Option<String>) -> impl IntoView {
    let notifications = expect_context::<NotificationStore>();
    Effect::new(move |_| notifications.mark_messages_read());
    let selected = conversation_id.unwrap_or_else(|| "hana".into());
    view! {
        <Shell active="dm" right_rail=false>
            <div class="wf-dm">
                <aside class="wf-dm-list flex flex-col">
                    <div class="flex items-center justify-between p-4 wf-spine-rule" style="margin:0;border-bottom:1px solid var(--line-soft);border-top:none;">
                        <span class="wf-title">"DM"</span>
                        <button class="wf-btn wf-btn-ghost wf-btn-sm wf-btn-circle">"+"</button>
                    </div>
                    <div class="p-3">
                        <input class="wf-input" style="padding:6px 10px;font-size:13px;" placeholder="検索" />
                    </div>
                    <div class="flex-1 overflow-y-auto">
                        {vec![
                            ("hana", "Hana K.", "@hana", "了解しました！", "2m", true),
                            ("riku", "Riku M.", "@riku", "OK 送信しました", "14m", false),
                            ("aya", "Aya T.", "@aya", "ありがとうございます", "1h", false),
                            ("design", "Group design", "3人", "Ken: 確かに", "3h", false),
                        ].into_iter().map(|(id, name, handle, last, time, unread)| {
                            let active = selected == id;
                            let row_style = if active { "background:var(--paper-2);" } else { "" };
                            view! {
                                <A href=format!("/dm/{id}") attr:class="flex items-center gap-3 p-3 cursor-pointer wf-spine-rule" attr:style=format!("margin:0;border-radius:0;border-top:none;border-bottom:1px dashed var(--line-soft);{row_style}")>
                                    <div class="avatar" style="width:40px;height:40px;flex-shrink:0;">
                                        <span class="font-bold" style="font-family:var(--font-hand);font-size:16px;">{name.chars().next().unwrap_or('?').to_string()}</span>
                                    </div>
                                    <div class="flex-1 min-w-0">
                                        <div class="flex items-center justify-between">
                                            <span class="font-bold text-sm truncate">{name}</span>
                                            <span class="wf-notif-time">{time}</span>
                                        </div>
                                        <div class="text-xs truncate wf-entry-meta">{handle} "  " {last}</div>
                                    </div>
                                    <Show when=move || unread>
                                        <span class="wf-badge" style="margin-left:0;" />
                                    </Show>
                                </A>
                            }
                        }).collect_view()}
                    </div>
                </aside>
                <main class="wf-dm-conv">
                    <div class="wf-dm-win-head">
                        <div class="avatar" style="width:36px;height:36px;">
                            <span class="font-bold" style="font-family:var(--font-hand);font-size:14px;">"H"</span>
                        </div>
                        <div class="flex flex-col">
                            <span class="font-bold text-sm">"Hana K."</span>
                            <span class="wf-notif-time">"@hana  オンライン"</span>
                        </div>
                        <button class="wf-btn wf-btn-ghost wf-btn-sm wf-btn-circle ml-auto">"···"</button>
                    </div>
                    <div class="wf-dm-msgs">
                        <span class="wf-entry-meta" style="text-align:center;">"── 今日 ──"</span>
                        <MessageBubble mine=false text="ブロックを解除してもいいですか？" />
                        <MessageBubble mine=true text="ブロックを解除すると、フォローも解除されます。" />
                        <MessageBubble mine=false text="了解しました、そのままでお願いします" />
                    </div>
                    <div class="p-3" style="border-top:1px solid var(--line-soft);">
                        <div class="flex gap-2 items-center">
                            <button class="wf-btn wf-btn-ghost wf-btn-sm wf-btn-circle">
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"/></svg>
                            </button>
                            <input class="wf-input flex-1" style="padding:8px 12px;" placeholder="メッセージを送信" />
                            <button class="wf-btn wf-btn-primary wf-btn-sm">"送信"</button>
                        </div>
                    </div>
                </main>
            </div>
        </Shell>
    }
}

#[component]
fn MessageBubble(#[prop(default = false)] mine: bool, #[prop(into)] text: String) -> impl IntoView {
    let class = if mine { "wf-bubble mine" } else { "wf-bubble" };
    view! { <div class=class>{text}</div> }
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

    let toggle_follow = move |_| {
        if follow_busy.get_untracked() {
            return;
        }
        let (Some(tok), Some(target)) = (token.get_untracked(), user.get_untracked()) else {
            return;
        };
        follow_busy.set(true);
        let currently = is_following.get_untracked();
        wasm_bindgen_futures::spawn_local(async move {
            let result = if currently {
                crate::api::users::unfollow(&tok, &target.id).await
            } else {
                crate::api::users::follow(&tok, &target.id).await
            };
            follow_busy.set(false);
            match result {
                Ok(()) => is_following.set(!currently),
                Err(e) => web_sys::console::error_1(&e.to_string().into()),
            }
        });
    };

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
                                        <button class="wf-btn wf-btn-primary wf-btn-sm" disabled=move || follow_busy.get() on:click=toggle_follow>
                                            {move || if is_following.get() { "フォロー中" } else { "フォロー" }}
                                        </button>
                                    </Show>
                                    <button class="wf-btn wf-btn-ghost wf-btn-sm wf-btn-circle">"···"</button>
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
                        class=move || if profile_tab.get() == "replies" { "active" } else { "" }
                        on:click=move |_| profile_tab.set("replies")
                        style="cursor:pointer">
                        "返信"
                    </a>
                    <a
                        class=move || if profile_tab.get() == "media" { "active" } else { "" }
                        on:click=move |_| profile_tab.set("media")
                        style="cursor:pointer">
                        "メディア"
                    </a>
                    <a
                        class=move || if profile_tab.get() == "likes" { "active" } else { "" }
                        on:click=move |_| profile_tab.set("likes")
                        style="cursor:pointer">
                        "いいね"
                    </a>
                </div>

                <div class="flex flex-col gap-3 px-4 mt-3">
                    {move || match profile_tab.get() {
                        "notes" => view! {
                            <For
                                each=move || notes.get()
                                key=|note| note.id.clone()
                                children=|note| view! { <PostCard note=note /> }
                            />
                        }.into_any(),
                        _ => view! { <div class="wf-empty">"[ COMING SOON ]"</div> }.into_any(),
                    }}
                </div>
            </section>
        </Shell>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let token = auth.token;
    let me = auth.me;

    let params = use_params_map();
    let section = move || {
        params
            .read()
            .get("section")
            .unwrap_or_else(|| "プロフィール".into())
    };

    let display_name_signal = RwSignal::new(String::new());
    let bio_signal = RwSignal::new(String::new());
    let handle_signal = RwSignal::new(String::new());

    // ユーザー情報がロードされたら初期化
    Effect::new(move |_| {
        if let Some(user) = me.get() {
            display_name_signal.set(user.display_name.clone().unwrap_or_default());
            bio_signal.set(user.bio.clone().unwrap_or_default());
            handle_signal.set(user.handle());
        }
    });

    let on_save = move |_| {
        let Some(tok) = token.get_untracked() else {
            return;
        };
        let req = crate::api::users::UpdateProfileRequest {
            display_name: Some(display_name_signal.get_untracked()),
            bio: Some(bio_signal.get_untracked()),
        };
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::users::update_me(&tok, &req).await {
                Ok(updated_user) => {
                    auth.me.set(Some(updated_user));
                }
                Err(e) => {
                    web_sys::console::error_1(&e.to_string().into());
                }
            }
        });
    };

    let groups = vec![
        (
            "アカウント",
            vec![
                "プロフィール",
                "セキュリティ",
                "パスワード",
                "連携アカウント",
                "2段階認証",
            ],
        ),
        ("プライバシー", vec!["公開範囲", "ブロック", "ミュート"]),
        ("通知", vec!["プッシュ", "メール", "デスクトップ"]),
        ("表示", vec!["テーマ", "フォント", "言語"]),
        ("連携", vec!["外部サービス", "API連携"]),
    ];

    let show_delete_confirm = RwSignal::new(false);
    let danger_open = RwSignal::new(false); // 危険な設定の折りたたみ

    let theme = RwSignal::new(
        LocalStorage::get::<String>("mithic.theme").unwrap_or_else(|_| "night".into()),
    );

    let set_theme = move |t: &'static str| {
        let actual_theme = match t {
            "dark" | "night" => "night",
            "light" => "light",
            "auto" => {
                let is_dark = web_sys::window()
                    .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
                    .map(|mql| mql.matches())
                    .unwrap_or(true);
                if is_dark { "night" } else { "light" }
            }
            _ => "night",
        };
        theme.set(t.into());
        let _ = LocalStorage::set("mithic.theme", t);
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Some(html) = doc.document_element() {
                let _ = html.set_attribute("data-theme", actual_theme);
            }
        }
    };

    view! {
        <Shell active="settings" right_rail=false>
            <div class="flex" style="height:100%;overflow:hidden;">
                <aside class="wf-rail" style="width:220px;flex-shrink:0;">
                    <span class="wf-title" style="font-size:18px;">"設定"</span>
                    {groups.into_iter().map(|(group, items)| view! {
                        <div>
                            <span class="wf-rail-tag" style="display:block;margin:8px 0 4px;">{group}</span>
                            {items.into_iter().map(|item| {
                                let is_active = move || section() == item;
                                view! {
                                    <A href=format!("/settings/{}", item)
                                        attr:class=move || if is_active() { "wf-pop-item active" } else { "wf-pop-item" }>
                                        {item}
                                    </A>
                                }
                            }).collect_view()}
                        </div>
                    }).collect_view()}
                </aside>
                <main class="wf-scroll p-6" style="flex:1;">
                    {move || match section().as_str() {
                        "プロフィール" => view! {
                            <span class="wf-entry-meta">"アカウント / プロフィール"</span>
                            <h1 class="wf-title mt-1 mb-6">"プロフィール設定"</h1>
                            <div class="flex flex-col gap-4 max-w-md">
                                <div class="flex items-center gap-4">
                                    <div class="avatar" style="width:80px;height:80px;">
                                        <span class="font-bold" style="font-size:24px;">"M"</span>
                                    </div>
                                    <div class="flex flex-col gap-2">
                                        <button class="wf-btn wf-btn-ghost wf-btn-sm">"画像を変更"</button>
                                        <button class="wf-btn wf-btn-ghost wf-btn-sm">"削除"</button>
                                    </div>
                                </div>
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
                                    <button class="wf-btn wf-btn-ghost">"キャンセル"</button>
                                    <button class="wf-btn wf-btn-primary" on:click=on_save>"保存"</button>
                                </div>
                            </div>
                        }.into_any(),
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
                        "2段階認証" => view! {
                            <span class="wf-entry-meta">"アカウント / 2段階認証"</span>
                            <h1 class="wf-title mt-1 mb-6">"2段階認証設定"</h1>
                            <div class="wf-card max-w-md">
                                <p class="text-sm opacity-70 mb-4">
                                    "2段階認証を有効にすると、ログイン時にパスワードに加えて認証コードの入力が必要になります。"
                                </p>
                                <div class="flex items-center gap-3">
                                    <button
                                        class="wf-btn wf-btn-primary wf-btn-sm"
                                        on:click=move |_| {
                                            let tok = token.get_untracked();
                                            let Some(tok) = tok else { return; };
                                            wasm_bindgen_futures::spawn_local(async move {
                                                use crate::api::auth;
                                                match auth::setup_2fa(&tok).await {
                                                    Ok(resp) => {
                                                        if let Some(w) = web_sys::window() {
                                                            _ = w.alert_with_message(
                                                                &format!("シークレット: {}\nこのコードを認証アプリに入力してください。", resp.secret)
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        if let Some(w) = web_sys::window() {
                                                            _ = w.alert_with_message(&e.to_string());
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    >
                                        "2段階認証を有効にする"
                                    </button>
                                </div>
                            </div>
                        }.into_any(),
                        other => view! {
                            <span class="wf-entry-meta">{format!("設定 / {other}")}</span>
                            <h1 class="wf-title mt-1 mb-6">{format!("{other}設定")}</h1>
                            <div class="wf-dashed p-8 text-center max-w-md">
                                <span class="wf-entry-meta">{format!("{other}に関する設定は現在準備中です。デフォルトで最適化されています。")}</span>
                            </div>
                        }.into_any()
                    }}

                    // 危険な設定の折りたたみ
                    <div class="wf-card max-w-md mt-6" style="border-color:var(--err);">
                        <button
                            class="flex items-center justify-between w-full text-sm font-bold"
                            style="color:var(--err);"
                            on:click=move |_| danger_open.update(|v| *v = !*v)
                        >
                            "危険な設定"
                            <span>{move || if danger_open.get() { "▲" } else { "▼" }}</span>
                        </button>
                        <Show when=move || danger_open.get()>
                            <div class="flex flex-col gap-3 mt-3">
                                <p class="text-xs opacity-70">
                                    "これらの操作は取り消せません。慎重に行ってください。"
                                </p>
                                <div class="flex gap-2">
                                    <button class="wf-btn wf-btn-ghost wf-btn-sm">"アカウントを一時停止"</button>
                                    <button class="wf-btn wf-btn-danger wf-btn-sm"
                                        on:click=move |_| show_delete_confirm.set(true)>
                                        "アカウントを削除する"
                                    </button>
                                </div>
                            </div>
                        </Show>
                    </div>

                    // 削除確認ダイアログ
                    <Show when=move || show_delete_confirm.get()>
                        <div class="wf-overlay" on:click=move |_| show_delete_confirm.set(false)>
                            <div class="wf-modal" style="max-width:380px;" on:click=move |e| e.stop_propagation()>
                                <div class="wf-modal-body">
                                    <h3 class="wf-modal-title" style="color:var(--err);font-size:18px;">"アカウントを削除しますか？"</h3>
                                    <p class="py-4 text-sm opacity-70">
                                        "この操作は取り消せません。投稿・フォロー関係・すべてのデータが完全に削除されます。"
                                    </p>
                                </div>
                                <div class="wf-modal-foot">
                                    <button class="wf-btn wf-btn-ghost" on:click=move |_| show_delete_confirm.set(false)>"キャンセル"</button>
                                    <button class="wf-btn wf-btn-danger">"削除する"</button>
                                </div>
                            </div>
                        </div>
                    </Show>
                </main>
            </div>
        </Shell>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let handle = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let remember = RwSignal::new(false);
    let show_pw = RwSignal::new(false);
    let error = RwSignal::<Option<String>>::new(None);
    let loading = RwSignal::new(false);
    let requires_2fa = RwSignal::new(false);
    let temp_token = RwSignal::new(String::new());
    let twofa_code = RwSignal::new(String::new());
    let navigate = use_navigate();
    let host = move || {
        web_sys::window()
            .and_then(|w| w.location().host().ok())
            .filter(|h| !h.is_empty())
            .unwrap_or_else(|| "mithic.social".into())
    };

    let on_submit = move |_| {
        let h = handle.get();
        let p = password.get();

        if h.trim().is_empty() {
            error.set(Some(
                "ユーザー名またはメールアドレスを入力してください".into(),
            ));
            return;
        }

        if requires_2fa.get_untracked() {
            let code = twofa_code.get_untracked();
            if code.is_empty() {
                error.set(Some("認証コードを入力してください".into()));
                return;
            }
            let temp_tok = temp_token.get_untracked();
            error.set(None);
            loading.set(true);
            let auth2 = auth.clone();
            let nav2 = navigate.clone();
            wasm_bindgen_futures::spawn_local(async move {
                use crate::api::auth::verify_2fa_signin;
                match verify_2fa_signin(&h, &temp_tok, &code).await {
                    Ok(resp) => {
                        auth2.login(resp.token, resp.user);
                        nav2("/", Default::default());
                    }
                    Err(e) => {
                        error.set(Some(e.message));
                        loading.set(false);
                    }
                }
            });
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
        wasm_bindgen_futures::spawn_local(async move {
            use crate::api::auth::login;
            let req = crate::api::auth::LoginRequest {
                username: h,
                password: p,
                remember: remember.get(),
            };
            match login(&req).await {
                Ok(pair) => {
                    if pair.requires_2fa.unwrap_or(false) {
                        loading.set(false);
                        requires_2fa.set(true);
                        temp_token.set(pair.temp_token.unwrap_or_default());
                        return;
                    }
                    auth2.login(pair.token, pair.user);
                    nav2("/", Default::default());
                }
                Err(e) => {
                    error.set(Some(e.message));
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="wf-auth">
            <aside class="wf-auth-aside">
                <span class="wf-mark wf-mark-lg">"[m]"<span class="br">"mithic"</span></span>
                <span class="wf-entry-meta mt-4" style="text-transform:uppercase;letter-spacing:0.1em;">"[ LOG IN  01 ]"</span>
                <h1 class="wf-auth-title mt-4">
                    "ようこそ、"<br />
                    <span class="br">"mithic"</span>"。"
                </h1>
                <p class="wf-auth-sub">"あなたの物語を、ここから続けましょう。"</p>
                <div class="wf-entry-meta mt-12">
                    "── mithic · signal not noise ──"
                </div>
            </aside>

            <div class="wf-auth-form">
                <div class="wf-auth-inner">
                    <span class="wf-mark wf-mark-md">"[m]"<span class="br">"mithic"</span></span>

                    <span class="wf-entry-meta">"[ 既存アカウント / SIGN IN ]"</span>
                    <h2 class="wf-auth-title" style="font-size:28px;">"ログイン"</h2>

                    <Show when=move || error.get().is_some()>
                        <div class="wf-alert error">
                            <span>{move || error.get().unwrap_or_default()}</span>
                        </div>
                    </Show>

                    <div class="flex flex-col gap-4 mt-4">
                        <div>
                            <div class="flex justify-between items-center mb-1">
                                <span class="wf-entry-meta">"サーバー"</span>
                                <span class="text-xs opacity-60 cursor-pointer hover:underline">"変更 ▸"</span>
                            </div>
                            <div class="wf-input flex items-center justify-between">
                                <div class="flex items-center gap-1">
                                    <span class="wf-entry-meta">"@"</span>
                                    <span>{host()}</span>
                                </div>
                                <span class="wf-pill on">"✓ 接続中"</span>
                            </div>
                        </div>

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

                        // 2FAコード入力 (2FA有効なアカウント用)
                        <Show when=move || requires_2fa.get()>
                            <div class="flex flex-col gap-2 mt-2">
                                <span class="text-xs font-bold" style="color:var(--warn);">"2段階認証コードを入力してください"</span>
                                <input
                                    class="wf-input"
                                    placeholder="000000"
                                    prop:value=move || twofa_code.get()
                                    on:input=move |e| twofa_code.set(event_target_value(&e))
                                />
                            </div>
                        </Show>

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
                </div>
            </div>
        </div>
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
        busy.set(true);
        error.set(None);
        let auth = auth.clone();
        let navigate = navigate.clone();
        wasm_bindgen_futures::spawn_local(async move {
            use crate::api::auth::{RegisterRequest, register};
            let request = RegisterRequest {
                username: signup_handle.get_untracked(),
                display_name: Some(display_name.get_untracked()).filter(|s| !s.is_empty()),
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
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    // ハンドル可用性チェック (簡易デバウンス)
    Effect::new(move |_| {
        let h = signup_handle.get();
        if h.len() < 3 {
            handle_available.set(None);
            return;
        }
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;
            if signup_handle.get_untracked() != h {
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

    let can_proceed = Memo::new(move |_| {
        handle_available.get() == Some(true)
            && !display_name.get().is_empty()
            && email.get().contains('@')
            && password.get().len() >= 8
            && password.get() == password_confirm.get()
            && agreed_age.get()
            && agreed_tos.get()
    });

    view! {
        <div class="wf-auth">
            <aside class="wf-auth-aside">
                <span class="wf-mark wf-mark-lg">"[m]"<span class="br">"mithic"</span></span>
                <span class="wf-entry-meta mt-4" style="text-transform:uppercase;letter-spacing:0.1em;">"[ SIGN UP  01 ]"</span>
                <h1 class="wf-auth-title mt-4">"アカウントを作成しましょう。"</h1>
                <p class="wf-auth-sub">"mithic はオープンな分散型 SNS です。ActivityPub でつながります。"</p>
                <div class="wf-entry-meta mt-12">
                    "── mithic · signal not noise ──"
                </div>
            </aside>

            <div class="wf-auth-form">
                <div class="wf-auth-inner">
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

                        <p class="text-xs text-center opacity-60 mt-4">
                            "既にアカウントをお持ちの方は "
                            <A href="/login" attr:class="font-bold" attr:style="color:var(--accent);">"ログイン →"</A>
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let kpis = vec![
        ("USERS", "1,204"),
        ("NOTES / DAY", "342"),
        ("FEDERATED", "58"),
        ("REPORTS", "3"),
    ];
    let bars = vec![30, 55, 40, 70, 65, 90, 50];
    view! {
        <Shell active="settings">
            <TopBar title="管理コンソール" folio="99" />
            <section class="wf-scroll">
                <div class="wf-admin-grid">
                    {kpis.into_iter().map(|(label, value)| view! {
                        <div class="wf-kpi">
                            <div class="v">{value}</div>
                            <div class="l">{label}</div>
                        </div>
                    }).collect_view()}
                </div>
                <div class="wf-card mx-4 mb-4">
                    <span class="wf-entry-meta">"[ NOTES / WEEK ]"</span>
                    <div class="wf-bars">
                        {bars.into_iter().map(|h| view! {
                            <div class="wf-bar" style=format!("height:{}%;", h) />
                        }).collect_view()}
                    </div>
                </div>
                <p class="wf-entry-meta px-4 pb-4">"管理機能は Phase 2 で実装予定です。"</p>
            </section>
        </Shell>
    }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <Shell active="home" right_rail=false>
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

#[component]
pub fn WelcomePage() -> impl IntoView {
    view! {
        <Shell active="home" right_rail=false>
            <section class="p-4 flex flex-col items-center justify-center min-h-[60dvh]">
                <div class="wf-card max-w-lg text-center flex flex-col items-center gap-6">
                    <span class="wf-mark wf-mark-lg">"[m]"<span class="br">"mithic"</span></span>
                    <span class="wf-entry-meta">"[ WELCOME ]"</span>
                    <h1 class="wf-title" style="font-size:30px;">"ご登録ありがとうございます"</h1>
                    <p class="text-sm opacity-70">
                        "mithic はオープンな分散型 SNS です。"
                        <br />
                        "ActivityPub で他のサーバーのユーザーとつながりましょう。"
                    </p>
                    <div class="flex flex-col gap-3 w-full max-w-sm">
                        <A href="/" attr:class="wf-btn wf-btn-primary" attr:style="width:100%;">
                            "ホームを見る"
                        </A>
                        <A href="/settings/プロフィール" attr:class="wf-btn wf-btn-ghost" attr:style="width:100%;">
                            "プロフィールを設定"
                        </A>
                    </div>
                </div>
            </section>
        </Shell>
    }
}
