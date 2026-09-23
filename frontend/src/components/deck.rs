use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_query_map};

use super::avatar::{Avatar, AvatarSize};
use super::load_more::LoadMore;
use super::markdown::MarkdownText;
use super::post_card::PostCard;
use crate::store::{AuthStore, NotificationStore, StreamStore};
use crate::time::relative_label;
use shared::{Note, Notification, NotificationType, User};

#[derive(Clone, Copy)]
pub enum TimelineKind {
    Home,
    Local,
    Global,
}

impl TimelineKind {
    fn api(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Local => "local",
            Self::Global => "global",
        }
    }
}

#[component]
pub fn TimelineColumn(kind: TimelineKind) -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let stream = expect_context::<StreamStore>();
    let notes = RwSignal::<Vec<Note>>::new(vec![]);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);
    let kind_str = kind.api();

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

    Effect::new(move |_| {
        let Some(note) = stream.latest_note.get() else {
            return;
        };
        let me_id = auth.me.get_untracked().map(|u| u.id);
        let accept = match kind {
            TimelineKind::Local => note.author.host.is_none(),
            TimelineKind::Global => true,
            TimelineKind::Home => {
                me_id.as_deref() == Some(note.author.id.as_str())
                    || notes
                        .with_untracked(|items| items.iter().any(|n| n.author.id == note.author.id))
            }
        };
        if !accept {
            return;
        }
        notes.update(|items| {
            if !items.iter().any(|n| n.id == note.id) {
                items.insert(0, note);
            }
        });
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
        <div class="wf-scroll">
            <Show when=move || is_loading.get() && notes.get().is_empty()>
                <div class="deck-status">
                    <span class="wf-spinner" />
                    <span class="wf-entry-meta">"読み込み中…"</span>
                </div>
            </Show>
            <Show when=move || !is_loading.get() && notes.get().is_empty()>
                <div class="wf-empty">
                    <span>"まだ投稿がありません"</span>
                </div>
            </Show>
            <For
                each=move || stream.visible(notes.get())
                key=|note| note.id.clone()
                children=|note| view! { <PostCard note=note /> }
            />
            <Show when=move || is_loading.get() && !notes.get().is_empty()>
                <div class="deck-status">
                    <span class="wf-spinner" />
                </div>
            </Show>
            <Show when=move || !is_loading.get() && has_more.get() && !notes.get().is_empty()>
                <LoadMore on_visible=std::sync::Arc::new(move || load_more()) />
            </Show>
        </div>
    }
}

#[component]
pub fn NotificationsColumn() -> impl IntoView {
    let notification_store = expect_context::<NotificationStore>();
    let auth = expect_context::<AuthStore>();
    let token = auth.token;
    let notifications = RwSignal::<Vec<Notification>>::new(vec![]);
    let filter = RwSignal::new("all");

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

    let filtered = move || {
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
        <div class="wf-scroll">
            <div class="deck-notif-bar">
                <div class="wf-chips">
                    <button
                        class=move || if filter.get() == "all" { "wf-chip on" } else { "wf-chip" }
                        on:click=move |_| filter.set("all")
                    >"すべて"</button>
                    <button
                        class=move || if filter.get() == "mention" { "wf-chip on" } else { "wf-chip" }
                        on:click=move |_| filter.set("mention")
                    >"返信"</button>
                    <button
                        class=move || if filter.get() == "reaction" { "wf-chip on" } else { "wf-chip" }
                        on:click=move |_| filter.set("reaction")
                    >"リアクション"</button>
                    <button
                        class=move || if filter.get() == "follow" { "wf-chip on" } else { "wf-chip" }
                        on:click=move |_| filter.set("follow")
                    >"フォロー"</button>
                </div>
                <button
                    class="wf-ico-btn"
                    on:click=mark_all_read
                    aria-label="すべて既読にする"
                    title="すべて既読にする"
                >
                    <Icon icon=id::FiCheckCircle width="16" height="16" />
                </button>
            </div>
            <For
                each=filtered
                key=|n| n.id.clone()
                children=|notification| {
                    let sender = notification.sender.clone();
                    let note = notification.note.clone();
                    let unread_class = if notification.is_read { "wf-notif" } else { "wf-notif unread" };
                    let kind_label = notif_label(&notification);
                    let when = relative_label(&notification.created_at);
                    view! {
                        <article class=unread_class>
                            {sender.map(|user| view! { <Avatar user=user size=AvatarSize::Sm /> }).into_view()}
                            <div class="wf-notif-text">
                                <div class="wf-notif-row">
                                    <span class="who">{kind_label}</span>
                                    <span class="wf-notif-time">{when}</span>
                                </div>
                                {note.map(|n| view! {
                                    <blockquote class="wf-notif-quote"><MarkdownText text=n.content /></blockquote>
                                }).into_view()}
                            </div>
                        </article>
                    }
                }
            />
            <Show when=move || filtered().is_empty()>
                <div class="wf-empty">
                    <span>"通知はまだありません"</span>
                </div>
            </Show>
        </div>
    }
}

fn notif_label(notification: &Notification) -> String {
    let who = notification
        .sender
        .as_ref()
        .map(|u| u.name())
        .unwrap_or_else(|| "誰か".into());
    match notification.notification_type {
        NotificationType::Reaction => format!(
            "{} がリアクション {}",
            who,
            notification.reaction.as_deref().unwrap_or("")
        ),
        NotificationType::Reply => format!("{who} が返信しました"),
        NotificationType::Follow => format!("{who} がフォローしました"),
        NotificationType::Renote => format!("{who} がリノートしました"),
        NotificationType::Mention => format!("{who} がメンションしました"),
        NotificationType::Quote => format!("{who} が引用しました"),
        NotificationType::FollowRequest => format!("{who} がフォローリクエスト"),
        NotificationType::FollowRequestAccepted => format!("{who} がリクエストを承認"),
        NotificationType::PollEnded => "アンケートが終了しました".into(),
        NotificationType::UserSignup => format!("{who} が登録しました"),
    }
}

#[component]
pub fn SearchColumn() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let query = use_query_map();
    let navigate = use_navigate();
    let search_input = RwSignal::new(String::new());
    let notes = RwSignal::<Vec<Note>>::new(Vec::new());
    let users = RwSignal::<Vec<User>>::new(Vec::new());
    let trend_tags = RwSignal::<Vec<shared::Hashtag>>::new(Vec::new());
    let loading = RwSignal::new(false);
    let searched = RwSignal::new(false);

    Effect::new(move |_| {
        search_input.set(query.read().get("q").unwrap_or_default());
    });

    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(tags) = crate::api::notes::fetch_trending(6).await {
                trend_tags.set(tags);
            }
        });
    });

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
                match crate::api::notes::fetch_hashtag_timeline(tok.as_deref(), &tag_val, 30).await {
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

    let go = move || {
        let q = crate::api::client::urlencoding_loose(&search_input.get());
        navigate(&format!("/search?q={q}"), Default::default());
    };

    view! {
        <div class="wf-scroll">
            <form
                class="deck-search-bar"
                on:submit=move |ev| {
                    ev.prevent_default();
                    go();
                }
            >
                <input
                    class="wf-input"
                    placeholder="投稿・ユーザー・タグ"
                    prop:value=move || search_input.get()
                    on:input=move |ev| search_input.set(event_target_value(&ev))
                    aria-label="検索"
                />
                <button class="wf-btn wf-btn-primary wf-btn-sm" type="submit">"検索"</button>
            </form>
            <Show when=move || !trend_tags.get().is_empty()>
                <div class="deck-search-tags">
                    {move || trend_tags.get().into_iter().map(|h| {
                        let bare = crate::api::client::urlencoding_loose(h.tag.trim_start_matches('#'));
                        let label = h.tag.clone();
                        view! { <A href=format!("/search?tag={bare}") attr:class="wf-pill">{label}</A> }
                    }).collect_view()}
                </div>
            </Show>
            <Show when=move || loading.get()>
                <div class="deck-status">
                    <span class="wf-spinner" />
                    <span class="wf-entry-meta">"検索中…"</span>
                </div>
            </Show>
            <Show when=move || !loading.get() && !users.get().is_empty()>
                <div class="deck-search-users">
                    {move || users.get().into_iter().map(|u| {
                        let href = format!("/profile/{}", u.route_handle());
                        let name = u.name();
                        let handle = u.handle();
                        view! {
                            <A href=href attr:class="wf-notif">
                                <Avatar user=u size=AvatarSize::Sm />
                                <div class="wf-notif-text">
                                    <div class="who">{name}</div>
                                    <div class="wf-entry-meta">{handle}</div>
                                </div>
                            </A>
                        }
                    }).collect_view()}
                </div>
            </Show>
            {move || {
                if loading.get() {
                    return ().into_any();
                }
                let list = notes.get();
                if !searched.get() {
                    view! {
                        <div class="wf-empty"><span>"キーワードまたはタグで検索できます。"</span></div>
                    }.into_any()
                } else if list.is_empty() && users.get().is_empty() {
                    view! {
                        <div class="wf-empty"><span>"検索結果が見つかりませんでした。"</span></div>
                    }.into_any()
                } else {
                    list.into_iter().map(|note| view! { <PostCard note=note /> }).collect_view().into_any()
                }
            }}
        </div>
    }
}
