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
    ("フォロー中", "/"),
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
    let active = match kind {
        TimelineKind::Home => "/",
        TimelineKind::Local => "/local",
        TimelineKind::Global => "/global",
    };
    let title = match kind {
        TimelineKind::Home => "�z�[��",
        TimelineKind::Local => "���[�J��",
        TimelineKind::Global => "�O���[�o��",
    };

    // �^�C�����C���ǂݍ���
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

    // WebSocket でリアルタイ�?先�?�挿入
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
            <TopBar title=title folio="01" tabs=TIMELINE_TABS.to_vec() active_tab=active />
            <section class="timeline-scroll">
                <For
                    each=move || notes.get()
                    key=|note| note.id.clone()
                    children=|note| view! { <PostCard note=note /> }
                />
                <Show when=move || is_loading.get()>
                    <div class="timeline-loading text-center py-4 flex justify-center">
                        <span class="loading loading-spinner loading-md"></span>
                        <span class="ml-2">"読み込み中…"</span>
                    </div>
                </Show>
                <Show when=move || !is_loading.get() && has_more.get() && !notes.get().is_empty()>
                    <LoadMore on_visible=move || load_more() />
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
    view! {
        <Shell active="home">
            <TopBar title="投稿詳細" folio="02" />
            <div class="flex flex-col lg:flex-row gap-4 p-4 timeline-scroll">
                <section class="flex-1 flex flex-col gap-4">
                    <PostCard note=current.clone() flat=true />
                    <div class="card card-bordered bg-base-100 shadow p-4 flex flex-col gap-3">
                        <span class="text-xs font-mono opacity-50">"返信�? " {current.author.handle()}</span>
                        <textarea class="textarea textarea-bordered w-full resize-none" placeholder=format!("{} への返信", current.author.handle()) />
                        <div class="flex items-center justify-between">
                            <div class="flex items-center gap-2">
                                <button class="btn btn-ghost btn-sm">"添�?"</button>
                                <button class="btn btn-ghost btn-sm">"絵�?�?"</button>
                            </div>
                            <button class="btn btn-primary btn-sm">"返信"</button>
                        </div>
                    </div>
                    {notes.into_iter().skip(1).map(|note| view! { <PostCard note=note /> }).collect_view()}
                </section>
                <aside class="w-full lg:w-80 flex flex-col gap-4">
                    <section class="card card-bordered bg-base-100 shadow p-4">
                        <span class="text-xs font-mono opacity-50 mb-2">"[ REACTIONS ]"</span>
                        <div class="flex flex-wrap gap-2 mt-2">
                            {current.reactions.iter().map(|reaction| view! {
                                <span class="badge badge-secondary p-3 gap-1">{reaction.emoji.clone()} " " {reaction.count.to_string()}</span>
                            }).collect_view()}
                        </div>
                    </section>
                    <section class="card card-bordered bg-base-100 shadow p-4">
                        <span class="text-xs font-mono opacity-50 mb-2">"[ QUOTES ]"</span>
                        <p class="text-sm">"引用は�?投稿を埋め込み表示します�?"</p>
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

    // �? API から通知一覧を取�?
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
            <div class="flex items-center justify-between p-4 border-b border-base-300">
                <span class="font-bold text-xl">"アク�?ィビティ"</span>
                <div class="flex items-center gap-2">
                    <Show when=move || (notification_store.unread_notifications.get() > 0)>
                        <span class="badge badge-error">
                            "未読 " {move || notification_store.unread_notifications.get().to_string()}
                        </span>
                    </Show>
                    <button class="btn btn-ghost btn-sm"
                        on:click=mark_all_read>
                        "既読に"
                    </button>
                </div>
            </div>
            <div class="tabs tabs-bordered px-4 mt-2">
                <span
                    class=move || if filter.get() == "all" { "tab tab-active" } else { "tab" }
                    on:click=move |_| filter.set("all")
                    style="cursor:pointer">
                    "すべて"
                </span>
                <span
                    class=move || if filter.get() == "mention" { "tab tab-active" } else { "tab" }
                    on:click=move |_| filter.set("mention")
                    style="cursor:pointer">
                    "@メンション"
                </span>
                <span
                    class=move || if filter.get() == "reaction" { "tab tab-active" } else { "tab" }
                    on:click=move |_| filter.set("reaction")
                    style="cursor:pointer">
                    "�?�?ね"
                </span>
                <span
                    class=move || if filter.get() == "follow" { "tab tab-active" } else { "tab" }
                    on:click=move |_| filter.set("follow")
                    style="cursor:pointer">
                    "フォロー"
                </span>
            </div>
            <section class="timeline-scroll">
                <For
                    each=filtered_notifications
                    key=|notification| notification.id.clone()
                    children=|notification| {
                    let sender = notification.sender.clone();
                    let note   = notification.note.clone();
                    let unread_class = if notification.is_read { "notification-card" } else { "notification-card unread" };
                    let (kind_label, action_view) = match notification.notification_type {
                        NotificationType::Reaction => (
                            format!("{} があなた�?�投稿に", notification.reaction.as_deref().unwrap_or("リアクション")),
                            view! {}.into_any(),
                        ),
                        NotificationType::Reply => (
                            "が返信しました".into(),
                            view! {
                                <div class="flex items-center gap-2 mt-3">
                                    <button class="btn btn-ghost btn-sm">"返信"</button>
                                    <button class="btn btn-sm">"開く"</button>
                                </div>
                            }.into_any(),
                        ),
                        NotificationType::Follow => (
                            "があなたをフォローしました".into(),
                            view! {
                                <div class="mt-3">
                                    <button class="btn btn-primary btn-sm">"フォローバック"</button>
                                </div>
                            }.into_any(),
                        ),
                        NotificationType::Renote => (
                            "がリノ�?�トしました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::Mention => (
                            "があなたをメンションしました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::Quote => (
                            "があなたを引用しました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::FollowRequest => (
                            "がフォローリクエストを送りました".into(),
                            view! {
                                <div class="flex items-center gap-2 mt-3">
                                    <button class="btn btn-primary btn-sm">"承�?"</button>
                                    <button class="btn btn-ghost btn-sm">"拒否"</button>
                                </div>
                            }.into_any(),
                        ),
                        NotificationType::FollowRequestAccepted => (
                            "があなた�?�フォローリクエストを承認しました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::PollEnded => (
                            "のアンケートが終�?しました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::UserSignup => (
                            "が登録しました".into(),
                            view! {}.into_any(),
                        ),
                    };
                    view! {
                        <article class=unread_class>
                            <Show when=move || !notification.is_read>
                                <div class="unread-indicator" />
                            </Show>
                            {sender.map(|user| view! { <Avatar user=user size=AvatarSize::Sm /> }).into_view()}
                            <div class="flex-1">
                                <div class="flex items-center justify-between">
                                    <strong class="text-sm">{kind_label}</strong>
                                    <span class="font-mono text-xs opacity-60">{notification.created_at}</span>
                                </div>
{note.map(|n| view! {
                                     <blockquote class="blockquote bg-base-200 p-3 rounded-lg text-sm mt-2"><MarkdownText text=n.content /></blockquote>
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

    // queryの"q"の値で初期�?
    let search_input = RwSignal::new(query.read().get("q").unwrap_or_default());

    // クエリパラメータの変更を監視して入力�?に反映
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
            <TopBar title="検索 / 発�?" folio="04" />
            <section class="timeline-scroll p-4 flex flex-col gap-4">
                <div class="card card-bordered bg-base-100 shadow p-4 flex flex-col gap-3">
                    <span class="text-xs font-mono opacity-50">"検索"</span>
                    <div class="join w-full">
                        <input
                            class="input input-bordered join-item flex-1"
                            placeholder="投稿・ユーザー・タグを検索"
                            prop:value=move || search_input.get()
                            on:input=move |ev| search_input.set(event_target_value(&ev))
                            on:keydown=move |ev| {
                                if ev.key() == "Enter" {
                                    do_search_key();
                                }
                            }
                        />
                        <button class="btn btn-primary join-item" on:click=move |_| do_search_click()>"検索"</button>
                    </div>
                    <div class="flex flex-wrap gap-2 mt-2">
                        {vec!["#art", "#tech", "#books", "#music", "#food", "#photo"].into_iter().map(|tag| view! {
                            <A href=format!("/search?tag={}", tag.trim_start_matches('#')) attr:class="badge badge-outline hover:badge-primary p-3">{tag}</A>
                        }).collect_view()}
                    </div>
                </div>
                <div class="flex flex-col gap-3">
                    {move || {
                        let notes = filtered_notes();
                        if notes.is_empty() {
                            view! {
                                <div class="card card-bordered border-dashed bg-base-100 p-8 text-center">
                                    <span class="font-mono text-sm opacity-55">"検索結果が見つかりませんでした�?"</span>
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
            <div class="dm-layout">
                <aside class="dm-list-pane flex flex-col">
                    <div class="flex items-center justify-between p-4 border-b border-base-300">
                        <span class="font-bold text-xl">"DM"</span>
                        <button class="btn btn-circle btn-sm btn-ghost">"+"</button>
                    </div>
                    <div class="p-3 border-b border-base-300">
                        <input class="input input-bordered input-sm w-full" placeholder="検索" />
                    </div>
                    <div class="flex-1 overflow-y-auto">
                        {vec![
                            ("hana", "Hana K.", "@hana", "余白につ�?て話そう", "2m", true),
                            ("riku", "Riku M.", "@riku", "OK 送りました", "14m", false),
                            ("aya", "Aya T.", "@aya", "?��", "1h", false),
                            ("design", "Group · design", "3 人", "Ken: たしかに", "3h", false),
                        ].into_iter().map(|(id, name, handle, last, time, unread)| {
                            let active = selected == id;
                            let row_class = if active {
                                "flex items-center gap-3 p-3 bg-base-200 hover:bg-base-200 cursor-pointer border-b border-base-300/40"
                            } else {
                                "flex items-center gap-3 p-3 hover:bg-base-200/50 cursor-pointer border-b border-base-300/40"
                            };
                            view! {
                                <A href=format!("/dm/{id}") attr:class=row_class>
                                    <div class="avatar placeholder">
                                        <div class="bg-primary text-primary-content rounded-full w-10 h-10 flex items-center justify-center">
                                            <span class="text-sm font-bold">{name.chars().next().unwrap_or('?').to_string()}</span>
                                        </div>
                                    </div>
                                    <div class="flex-1 min-w-0">
                                        <div class="flex items-center justify-between">
                                            <span class="font-bold text-sm truncate">{name}</span>
                                            <span class="font-mono text-xs opacity-50">{time}</span>
                                        </div>
                                        <div class="text-xs truncate opacity-60">{handle} " · " {last}</div>
                                    </div>
                                    <Show when=move || unread>
                                        <div class="unread-indicator" />
                                    </Show>
                                </A>
                            }
                        }).collect_view()}
                    </div>
                </aside>
                <main class="dm-conversation">
                    <div class="flex items-center justify-between p-4 border-b border-base-300 bg-base-100">
                        <div class="flex items-center gap-3">
                            <div class="avatar placeholder">
                                <div class="bg-primary text-primary-content rounded-full w-10 h-10 flex items-center justify-center">
                                    <span class="text-sm font-bold">"H"</span>
                                </div>
                            </div>
                            <div class="flex flex-col">
                                <span class="font-bold text-sm">"Hana K."</span>
                                <span class="font-mono text-xs opacity-50">"@hana · オンライン"</span>
                            </div>
                        </div>
                        <button class="btn btn-ghost btn-circle">"···"</button>
                    </div>
                    <div class="dm-messages">
                        <span class="font-mono text-xs text-center opacity-40 my-2">"�? 今日 �?"</span>
                        <MessageBubble mine=false text="ワイヤーの粒度ってど�?決めてる�?" />
                        <MessageBubble mine=true text="決めすぎな�?ように。会話が生まれる粒度�?" />
                        <MessageBubble mine=false text="なるほど。じ�?あ余白につ�?て話そう" />
                    </div>
                    <div class="p-4 bg-base-100 border-t border-base-300">
                        <div class="join w-full">
                            <button class="btn join-item btn-outline border-base-300 bg-base-100">"?��"</button>
                            <input class="input input-bordered join-item flex-1" placeholder="メ�?セージを�?�力…" />
                            <button class="btn join-item btn-outline border-base-300 bg-base-100">"???"</button>
                        </div>
                    </div>
                </main>
            </div>
        </Shell>
    }
}

#[component]
fn MessageBubble(#[prop(default = false)] mine: bool, #[prop(into)] text: String) -> impl IntoView {
    let class = if mine {
        "message-bubble mine"
    } else {
        "message-bubble"
    };
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

    // プロフィールと投稿一覧を�? API から取�?
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
            <section class="timeline-scroll">
                <div class="relative">
                    <div class="profile-banner" />
                    <div class="profile-header flex flex-col p-4 relative border-b border-base-300">
                        <div class="profile-avatar-wrap absolute -top-11 left-4 z-10">
                            {move || user.get().map(|u| view! { <Avatar user=u size=AvatarSize::Xl /> })}
                        </div>
                        <div class="profile-meta pt-14 flex flex-col gap-2">
                            <div class="flex items-center justify-between">
                                <div>
                                    <span class="text-xs font-mono opacity-50">"ACTIVITYPUB · " {move || if handle().contains('@') { "REMOTE" } else { "LOCAL" }}</span>
                                    <h1 class="text-2xl font-bold mt-1">{move || user.get().map(|u| u.name()).unwrap_or_default()}</h1>
                                    <span class="font-mono text-sm opacity-60">{move || format!("@{}", handle())}</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <Show when=move || auth.me.get().zip(user.get()).map(|(me, u)| me.id != u.id).unwrap_or(false)>
                                        <button class="btn btn-primary btn-sm" disabled=move || follow_busy.get() on:click=toggle_follow>
                                            {move || if is_following.get() { "フォロー中" } else { "フォロー" }}
                                        </button>
                                    </Show>
                                    <button class="btn btn-ghost btn-circle btn-sm">"···"</button>
                                </div>
                            </div>
                            <p class="text-sm mt-2">{move || user.get().and_then(|u| u.bio).unwrap_or_default()}</p>

                            <div class="profile-stats flex gap-6 mt-4">
                                <div class="stat-item flex flex-col">
                                    <span class="stat-value text-lg font-bold">{move || user.get().map(|u| u.notes_count).unwrap_or(0).to_string()}</span>
                                    <span class="stat-label text-xs opacity-50 font-mono">"投稿"</span>
                                </div>
                                <div class="stat-item flex flex-col">
                                    <span class="stat-value text-lg font-bold">{move || user.get().map(|u| u.followers_count).unwrap_or(0).to_string()}</span>
                                    <span class="stat-label text-xs opacity-50 font-mono">"フォロワー"</span>
                                </div>
                                <div class="stat-item flex flex-col">
                                    <span class="stat-value text-lg font-bold">{move || user.get().map(|u| u.following_count).unwrap_or(0).to_string()}</span>
                                    <span class="stat-label text-xs opacity-50 font-mono">"フォロー"</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="tabs tabs-boxed mx-4 my-3">
                    <span
                        class=move || if profile_tab.get() == "notes" { "tab tab-active" } else { "tab" }
                        on:click=move |_| profile_tab.set("notes")
                        style="cursor:pointer">
                        "投稿"
                    </span>
                    <span
                        class=move || if profile_tab.get() == "replies" { "tab tab-active" } else { "tab" }
                        on:click=move |_| profile_tab.set("replies")
                        style="cursor:pointer">
                        "返信"
                    </span>
                    <span
                        class=move || if profile_tab.get() == "media" { "tab tab-active" } else { "tab" }
                        on:click=move |_| profile_tab.set("media")
                        style="cursor:pointer">
                        "メ�?ィア"
                    </span>
                    <span
                        class=move || if profile_tab.get() == "likes" { "tab tab-active" } else { "tab" }
                        on:click=move |_| profile_tab.set("likes")
                        style="cursor:pointer">
                        "�?�?ね"
                    </span>
                </div>

                <div class="flex flex-col gap-3 px-4">
                    {move || match profile_tab.get() {
                        "notes" => view! {
                            <For
                                each=move || notes.get()
                                key=|note| note.id.clone()
                                children=|note| view! { <PostCard note=note /> }
                            />
                        }.into_any(),
                        _ => view! { <div class="text-center py-8 opacity-50 font-mono text-sm">"[ COMING SOON ]"</div> }.into_any(),
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

    // ユーザー�?報がロードされたら�?�期�?
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
            "アカウン�?",
            vec![
                "プロフィール",
                "メール",
                "パスワー�?",
                "連携アカウン�?",
                "2段階認証",
            ],
        ),
        ("プライバシー", vec!["公開�?囲", "ブロ�?ク", "ミュー�?"]),
        ("通知", vec!["プッシュ", "メール", "メンション"]),
        ("表示", vec!["�?ー�?", "言�?", "�?度", "タイ�?ゾーン"]),
        ("連携", vec!["外部サービス", "APIト�?�クン"]),
    ];

    let show_delete_confirm = RwSignal::new(false);
    let danger_open = RwSignal::new(false); // 危険な設定�?�アコー�?ィオン

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
            <div class="settings-layout">
                <aside class="settings-nav">
                    <span class="font-bold text-lg px-2 block mb-4">"設�?"</span>
                    <ul class="menu w-full p-0 gap-2">
                        {groups.into_iter().map(|(group, items)| view! {
                            <li class="menu-title text-xs font-mono opacity-50 px-2 mt-2">{group}</li>
                            {items.into_iter().map(|item| {
                                let is_active = move || section() == item;
                                view! {
                                    <li>
                                        <A href=format!("/settings/{}", item)
                                            attr:class=move || if is_active() { "active" } else { "" }>
                                            {item}
                                        </A>
                                    </li>
                                }
                            }).collect_view()}
                        }).collect_view()}
                    </ul>
                </aside>
                <main class="settings-content">
                    {move || match section().as_str() {
                        "プロフィール" => view! {
                            <span class="text-xs font-mono opacity-50">"アカウン�? / プロフィール"</span>
                            <h1 class="text-2xl font-bold mt-1 mb-6">"プロフィール設�?"</h1>
                            <div class="flex flex-col gap-4 max-w-md">
                                <div class="flex items-center gap-4">
                                    <div class="avatar placeholder">
                                        <div class="bg-primary text-primary-content rounded-full w-20 h-20 flex items-center justify-center">
                                            <span class="text-2xl font-bold">"M"</span>
                                        </div>
                                    </div>
                                    <div class="flex flex-col gap-2">
                                        <button class="btn btn-sm btn-outline">"画像を変更"</button>
                                        <button class="btn btn-sm btn-ghost text-error">"削除"</button>
                                    </div>
                                </div>
                                <label class="form-control w-full">
                                    <div class="label">
                                        <span class="label-text text-xs font-mono opacity-50">"表示�?"</span>
                                    </div>
                                    <input
                                        class="input input-bordered w-full"
                                        prop:value=move || display_name_signal.get()
                                        on:input=move |ev| display_name_signal.set(event_target_value(&ev))
                                    />
                                </label>
                                <label class="form-control w-full">
                                    <div class="label">
                                        <span class="label-text text-xs font-mono opacity-50">"ハンドル"</span>
                                    </div>
                                    <input
                                        class="input input-bordered w-full"
                                        prop:value=move || handle_signal.get()
                                        disabled=true
                                    />
                                </label>
                                <label class="form-control w-full">
                                    <div class="label">
                                        <span class="label-text text-xs font-mono opacity-50">"自己紹�?"</span>
                                    </div>
                                    <textarea
                                        class="textarea textarea-bordered h-24 w-full resize-none"
                                        prop:value=move || bio_signal.get()
                                        on:input=move |ev| bio_signal.set(event_target_value(&ev))
                                    />
                                </label>
                                <div class="flex items-center justify-end gap-2 mt-4">
                                    <button class="btn btn-ghost">"キャンセル"</button>
                                    <button class="btn btn-primary" on:click=on_save>"保�?"</button>
                                </div>
                            </div>
                        }.into_any(),
                        "�?ー�?" => view! {
                            <span class="text-xs font-mono opacity-50">"表示 / �?ー�?"</span>
                            <h1 class="text-2xl font-bold mt-1 mb-6">"�?ーマ設�?"</h1>
                            <div class="card card-bordered bg-base-100 p-4 max-w-md flex flex-row items-center justify-between shadow">
                                <span class="text-sm font-semibold">"�?ー�?"</span>
                                <div class="join">
                                    <button
                                        class=move || if theme.get() == "light" { "btn btn-sm btn-primary join-item" } else { "btn btn-sm btn-ghost join-item" }
                                        on:click=move |_| set_theme("light")>
                                        "ライ�?"
                                    </button>
                                    <button
                                        class=move || if theme.get() == "dark" || theme.get() == "night" { "btn btn-sm btn-primary join-item" } else { "btn btn-sm btn-ghost join-item" }
                                        on:click=move |_| set_theme("night")>
                                        "ダーク"
                                    </button>
                                    <button
                                        class=move || if theme.get() == "auto" { "btn btn-sm btn-primary join-item" } else { "btn btn-sm btn-ghost join-item" }
                                        on:click=move |_| set_theme("auto")>
                                        "自�?"
                                    </button>
                                </div>
                            </div>
                        }.into_any(),
                        "2�i�K�F��" => view! {
                            <span class="text-xs font-mono opacity-50">"�A�J�E���g / 2�i�K�F��"</span>
                            <h1 class="text-2xl font-bold mt-1 mb-6">"2�i�K�F�ؐݒ�"</h1>
                            <div class="card card-bordered bg-base-100 p-6 max-w-md shadow">
                                <p class="text-sm opacity-70 mb-4">
                                    "2�i�K�F�؂�L���ɂ���ƁA���O�C�����Ƀp�X���[�h�ɉ����ĔF�؃R�[�h�̓��͂��K�v�ɂȂ�܂��B"
                                </p>
                                <div class="flex items-center gap-3">
                                    <button
                                        class="btn btn-primary btn-sm"
                                        on:click=move |_| {
                                            let tok = token.get_untracked();
                                            let Some(tok) = tok else { return; };
                                            wasm_bindgen_futures::spawn_local(async move {
                                                use crate::api::auth;
                                                match auth::setup_2fa(&tok).await {
                                                    Ok(resp) => {
                                                        web_sys::window()
                                                            .and_then(|w| w.alert_with_message(
                                                                &format!("�V�[�N���b�g: {}\n���̃R�[�h��F�؃A�v���ɓ��͂��Ă��������B", resp.secret)
                                                            ));
                                                    }
                                                    Err(e) => {
                                                        web_sys::window()
                                                            .and_then(|w| w.alert_with_message(&e.to_string()));
                                                    }
                                                }
                                            });
                                        }
                                    >
                                        "2�i�K�F�؂�L���ɂ���"
                                    </button>
                                </div>
                            </div>
                        }.into_any(),
                        other => view! {
                            <span class="text-xs font-mono opacity-50">{format!("設�? / {other}")}</span>
                            <h1 class="text-2xl font-bold mt-1 mb-6">{format!("{other}設�?")}</h1>
                            <div class="card card-bordered border-dashed bg-base-100 p-8 text-center max-w-md shadow">
                                <span class="font-mono text-sm opacity-55">{format!("{other}に関する設定�?�現在準備中か、デフォルトで最適化されて�?ます�?")}</span>
                            </div>
                        }.into_any()
                    }}

                    // 危険な操�? (DaisyUI collapse)
                    <div class="collapse collapse-arrow border border-error/20 bg-error/5 max-w-md mt-6 rounded-lg">
                        <input type="checkbox"
                            prop:checked=move || danger_open.get()
                            on:change=move |_| danger_open.update(|v| *v = !*v)
                        />
                        <div class="collapse-title text-sm font-bold text-error">
                            "危険な設�?"
                        </div>
                        <div class="collapse-content flex flex-col gap-3">
                            <p class="text-xs opacity-70">
                                "これら�?�操作�?�取り消せません。�?�重に行ってください�?"
                            </p>
                            <div class="flex gap-2">
                                <button class="btn btn-xs btn-outline">"アカウントを一時停止"</button>
                                <button class="btn btn-xs btn-error btn-outline"
                                    on:click=move |_| show_delete_confirm.set(true)>
                                    "アカウントを削除する"
                                </button>
                            </div>
                        </div>
                    </div>

                    // 削除確認ダイアログ
                    <dialog class="modal" class:modal-open=move || show_delete_confirm.get()>
                        <div class="modal-box max-w-sm border border-error/20">
                            <h3 class="font-bold text-lg text-error">"アカウントを削除しますか?�?"</h3>
                            <p class="py-4 text-sm opacity-70">
                                "こ�?�操作�?�取り消せません。投稿・フォロー関係�?�すべての�?ータが完�?�に削除されます�?"
                            </p>
                            <div class="modal-action">
                                <button class="btn btn-ghost" on:click=move |_| show_delete_confirm.set(false)>"キャンセル"</button>
                                <button class="btn btn-error">"削除する"</button>
                            </div>
                        </div>
                        <form method="dialog" class="modal-backdrop" on:submit=move |e| { e.prevent_default(); show_delete_confirm.set(false); }>
                            <button>"close"</button>
                        </form>
                    </dialog>
                </main>
            </div>
        </Shell>
    }
}

#[component]
fn Field(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <label class="form-control w-full">
            <div class="label">
                <span class="label-text text-xs font-mono opacity-50">{label}</span>
            </div>
            <input class="input input-bordered w-full" value=value />
        </label>
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
                "���[�U�[���܂��̓��[���A�h���X����͂��Ă�������".into(),
            ));
            return;
        }

        if requires_2fa.get_untracked() {
            let code = twofa_code.get_untracked();
            if code.is_empty() {
                error.set(Some("�F�؃R�[�h����͂��Ă�������".into()));
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
            error.set(Some("�p�X���[�h��8�����ȏ��".into()));
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
                    auth2.login(pair.access_token, pair.user);
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
        <div class="auth-split">
            <aside class="auth-aside">
                <div class="auth-aside-content">
                    <div class="auth-aside-logo">"mithic"</div>
                    <span class="text-xs font-mono opacity-60 uppercase tracking-widest">"[ LOG IN · 01 ]"</span>
                    <h1 class="auth-aside-title mt-4">
                        "ようこそ�?"<br />
                        <span class="underline underline-offset-4 decoration-primary">"mithic"</span>" へ�?"
                    </h1>
                    <p class="auth-aside-sub mt-2">"決めな�?自由を残したまま、�?�開しましょ�?�?"</p>
                    <div class="font-mono text-xs opacity-40 mt-12">
                        "�? mithic · signal not noise �?"
                    </div>
                </div>
            </aside>

            <div class="auth-form-area">
                <div class="auth-form-inner">
                    <div class="flex items-center gap-2 mb-4">
                        <span class="text-2xl font-bold text-primary">"[m]"</span>
                        <span class="font-bold text-2xl">"mithic"</span>
                    </div>

                    <span class="text-xs font-mono opacity-50">[ 既存アカウン�? / SIGN IN ]</span>
                    <h2 class="text-3xl font-extrabold mt-1">"ログイン"</h2>

                    <Show when=move || error.get().is_some()>
                        <div class="alert alert-error text-sm py-3 mt-2">
                            <span>{move || error.get().unwrap_or_default()}</span>
                        </div>
                    </Show>

                    <div class="flex flex-col gap-4 mt-4">
                        <div>
                            <div class="flex justify-between items-center mb-1">
                                <span class="text-xs font-mono opacity-50">"サー�?"</span>
                                <span class="text-xs opacity-60 cursor-pointer hover:underline">"変更 ▾"</span>
                            </div>
                            <div class="input input-bordered w-full flex items-center justify-between">
                                <div class="flex items-center gap-1">
                                    <span class="text-base-content/40 font-mono">"@"</span>
                                    <span>{host()}</span>
                                </div>
                                <span class="badge badge-success badge-sm">"�? 接続中"</span>
                            </div>
                        </div>

                        <label class="form-control w-full">
                            <div class="label">
                                <span class="label-text text-xs font-mono opacity-50">"ハンドル / メール"</span>
                            </div>
                            <input class="input input-bordered w-full"
                                placeholder="@hana"
                                prop:value=move || handle.get()
                                on:input=move |e| handle.set(event_target_value(&e))
                            />
                        </label>

                        <div>
                            <div class="flex justify-between items-center mb-1">
                                <span class="text-xs font-mono opacity-50">"パスワー�?"</span>
                                <span class="text-xs text-primary cursor-pointer hover:underline">"忘れた�?��?"</span>
                            </div>
                            <div class="input input-bordered w-full flex items-center justify-between">
                                <input
                                    class="flex-1 bg-transparent border-none outline-none text-base-content"
                                    prop:type=move || if show_pw.get() { "text" } else { "password" }
                                    placeholder="••••••••"
                                    prop:value=move || password.get()
                                    on:input=move |e| password.set(event_target_value(&e))
                                />
                                <button class="btn btn-ghost btn-xs btn-circle" on:click=move |_| show_pw.update(|v| *v = !*v)>
                                    {move || if show_pw.get() { "?��" } else { "?��" }}
                                </button>
                            </div>
                        </div>

                        <div class="flex items-center justify-between text-xs mt-2">
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="checkbox" class="checkbox checkbox-sm checkbox-primary"
                                    prop:checked=move || remember.get()
                                    on:change=move |e| remember.set(event_target_checked(&e))
                                />
                                <span>"���̃u���E�U���L��"</span>
                            </label>
                        </div>

                        // 2FA�R�[�h���� (2FA�L���ȃA�J�E���g�p)
                        <Show when=move || requires_2fa.get()>
                            <div class="flex flex-col gap-2 mt-2">
                                <span class="text-xs text-warning font-bold">"2�i�K�F�؃R�[�h����͂��Ă�������"</span>
                                <input
                                    class="input input-bordered input-sm w-full"
                                    placeholder="000000"
                                    prop:value=move || twofa_code.get()
                                    on:input=move |e| twofa_code.set(event_target_value(&e))
                                />
                            </div>
                        </Show>

                        <button class="btn btn-primary btn-block mt-4"
                            disabled=move || loading.get()
                            on:click=on_submit>
                            {move || if loading.get() { "�F�ؒ��c" } else { "���O�C�� ��" }}
                        </button>

                        <p class="text-xs text-center opacity-60 mt-4">
                            "はじめての方は "
                            <A href="/signup" attr:class="link link-primary font-bold">"新規登録 �?"</A>
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

    // 新規登録の�? API 呼び出�?
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
                handle: signup_handle.get_untracked(),
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

    // ハンドル可用性チェ�?ク (簡易デバウンス)
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
        <div class="auth-split">
            <aside class="auth-aside">
                <div class="auth-aside-content">
                    <div class="auth-aside-logo">"mithic"</div>
                    <span class="text-xs font-mono opacity-60 uppercase tracking-widest">"[ SIGN UP · 01 ]"</span>
                    <h1 class="auth-aside-title mt-4">"アカウントを作�?�しましょ�?�?"</h1>
                    <p class="auth-aside-sub mt-2">"mithic はオープンな�?散�? SNS です�?ActivityPub でつながります�?"</p>
                    <div class="font-mono text-xs opacity-40 mt-12">
                        "�? mithic · signal not noise �?"
                    </div>
                </div>
            </aside>

            <div class="auth-form-area">
                <div class="auth-form-inner">
                    <div class="signup-progress flex gap-1 h-1 bg-base-300 w-full rounded-full overflow-hidden">
                        <div class="signup-progress-seg bg-primary flex-1 h-full" />
                        <div class="signup-progress-seg bg-base-300 flex-1 h-full" />
                        <div class="signup-progress-seg bg-base-300 flex-1 h-full" />
                    </div>

                    <span class="text-xs font-mono opacity-50 mt-4">"[ STEP 1/3 · 登録�?報 ]"</span>
                    <h2 class="text-3xl font-extrabold mt-1">"新規登録"</h2>

                    <Show when=move || error.get().is_some()>
                        <div class="alert alert-error text-sm py-3 mt-2">
                            <span>{move || error.get().unwrap_or_default()}</span>
                        </div>
                    </Show>

                    <div class="flex flex-col gap-4 mt-4">
                        <div>
                            <div class="flex justify-between items-center mb-1">
                                <span class="text-xs font-mono opacity-50">"ハンドル"</span>
                                {move || match handle_available.get() {
                                    Some(true)  => view! { <span class="badge badge-success badge-sm">"�? 利用可"</span> }.into_any(),
                                    Some(false) => view! { <span class="badge badge-error badge-sm">"�? 使用中"</span> }.into_any(),
                                    None        => view! { <span></span> }.into_any(),
                                }}
                            </div>
                            <input class="input input-bordered w-full"
                                placeholder="@hana"
                                prop:value=move || signup_handle.get()
                                on:input=move |e| signup_handle.set(event_target_value(&e))
                            />
                        </div>

                        <label class="form-control w-full">
                            <div class="label">
                                <span class="label-text text-xs font-mono opacity-50">"表示�?"</span>
                            </div>
                            <input class="input input-bordered w-full"
                                placeholder="Hana K."
                                prop:value=move || display_name.get()
                                on:input=move |e| display_name.set(event_target_value(&e))
                            />
                        </label>

                        <label class="form-control w-full">
                            <div class="label">
                                <span class="label-text text-xs font-mono opacity-50">"メールアドレス"</span>
                            </div>
                            <input class="input input-bordered w-full"
                                type="email"
                                placeholder="hana@example.com"
                                prop:value=move || email.get()
                                on:input=move |e| email.set(event_target_value(&e))
                            />
                        </label>

                        <div>
                            <label class="form-control w-full">
                                <div class="label">
                                    <span class="label-text text-xs font-mono opacity-50">"パスワー�?"</span>
                                </div>
                                <input class="input input-bordered w-full"
                                    type="password"
                                    placeholder="••••••••"
                                    prop:value=move || password.get()
                                    on:input=move |e| password.set(event_target_value(&e))
                                />
                            </label>
                            <div class="pw-strength-bar flex gap-1 mt-2">
                                {move || (1..=4u8).map(|i| {
                                    let strength = pw_strength.get();
                                    let cls = if strength >= i {
                                        format!("pw-strength-seg active-{i} flex-1 h-1 rounded-full")
                                    } else {
                                        "pw-strength-seg flex-1 h-1 rounded-full bg-base-300".into()
                                    };
                                    view! { <div class=cls /> }
                                }).collect_view()}
                            </div>
                        </div>

                        <label class="form-control w-full">
                            <div class="label">
                                <span class="label-text text-xs font-mono opacity-50">"パスワード確�?"</span>
                            </div>
                            <input class="input input-bordered w-full"
                                type="password"
                                placeholder="••••••••"
                                prop:value=move || password_confirm.get()
                                on:input=move |e| password_confirm.set(event_target_value(&e))
                            />
                        </label>

                        <div class="flex flex-col gap-2 mt-2">
                            <label class="flex items-center gap-2 cursor-pointer text-xs">
                                <input type="checkbox" class="checkbox checkbox-sm checkbox-primary"
                                    prop:checked=move || agreed_age.get()
                                    on:change=move |e| agreed_age.set(event_target_checked(&e))
                                />
                                <span>"私�?�13歳以上で�?"</span>
                            </label>

                            <label class="flex items-center gap-2 cursor-pointer text-xs">
                                <input type="checkbox" class="checkbox checkbox-sm checkbox-primary"
                                    prop:checked=move || agreed_tos.get()
                                    on:change=move |e| agreed_tos.set(event_target_checked(&e))
                                />
                                <span>"利用規�?に同意しま�?"</span>
                            </label>
                        </div>

                        <button class="btn btn-primary btn-block mt-4"
                            disabled=move || !can_proceed.get() || busy.get()
                            on:click=do_register>
                            {move || if busy.get() { "登録中…" } else { "アカウント作�?? �?" }}
                        </button>

                        <p class="text-xs text-center opacity-60 mt-4">
                            "既にアカウントをお持ちの方は "
                            <A href="/login" attr:class="link link-primary font-bold">"ログイン �?"</A>
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <Shell active="settings">
            <TopBar title="管�?コンソール" folio="99" />
            <section class="p-4 timeline-scroll">
                <div class="card card-bordered bg-base-100 shadow p-6">
                    <h2 class="text-2xl font-bold mb-2">"Admin"</h2>
                    <p class="text-sm opacity-70">"P2で実�?予定�?�管�?画面です�?"</p>
                </div>
            </section>
        </Shell>
    }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <Shell active="home" right_rail=false>
            <section class="p-4 flex flex-col items-center justify-center min-h-[50dvh]">
                <div class="card card-bordered bg-base-100 shadow p-8 max-w-sm text-center flex flex-col items-center gap-4">
                    <span class="text-xs font-mono opacity-50">[ 404 ]</span>
                    <h1 class="text-2xl font-bold">"ペ�?�ジが見つかりません"</h1>
                    <A href="/" attr:class="btn btn-primary btn-block">"ホ�?��?へ戻�?"</A>
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
                <div class="card card-bordered bg-base-100 shadow p-8 max-w-lg text-center flex flex-col items-center gap-6">
                    <div class="auth-aside-logo text-3xl font-bold text-primary">"mithic"</div>
                    <span class="text-xs font-mono opacity-50">[ WELCOME ]</span>
                    <h1 class="text-3xl font-extrabold">"���o�^���肪�Ƃ��������܂�"</h1>
                    <p class="text-sm opacity-70">
                        "mithic �̓I�[�v���ȕ��U�^ SNS �ł��B"
                        <br />
                        "ActivityPub �ő��̃T�[�o�[�̃��[�U�[�ƂȂ���܂��傤�B"
                    </p>
                    <div class="flex flex-col gap-3 w-full max-w-sm">
                        <A href="/" attr:class="btn btn-primary btn-block">
                            "�z�[��������"
                        </A>
                        <A href="/settings/�v���t�B�[��" attr:class="btn btn-ghost btn-block">
                            "�v���t�B�[����ݒ�"
                        </A>
                    </div>
                </div>
            </section>
        </Shell>
    }
}
