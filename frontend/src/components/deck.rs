use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;

use super::avatar::{Avatar, AvatarSize};
use super::load_more::LoadMore;
use super::markdown::MarkdownText;
use super::post_card::PostCard;
use super::shell::Shell;
use crate::store::{AuthStore, ColumnKind, DeckStore, NotificationStore, StreamStore};
use crate::time::relative_label;
use shared::{Note, Notification, NotificationType};

#[component]
pub fn DeckPage(#[prop(into)] active: String) -> impl IntoView {
    let deck = expect_context::<DeckStore>();
    let location = use_location();
    let focus = Signal::derive(move || ColumnKind::from_path(&location.pathname.get()));
    let columns = Signal::derive(move || deck.visible_with_focus(focus.get()));
    let add_open = RwSignal::new(false);

    Effect::new(move |_| {
        let Some(kind) = focus.get() else {
            return;
        };
        let id = format!("deck-col-{}", kind.id());
        #[cfg(target_arch = "wasm32")]
        {
            gloo_timers::callback::Timeout::new(40, move || {
                if let Some(el) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id(&id))
                {
                    el.scroll_into_view();
                }
            })
            .forget();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = id;
        }
    });

    let missing = Signal::derive(move || deck.missing());
    let can_add = Signal::derive(move || !missing.get().is_empty());

    view! {
        <Shell active=active deck=true>
            <div class="deck-board" aria-label="タイムライン">
                <For
                    each=move || columns.get()
                    key=|kind| kind.id()
                    children=move |kind| view! { <DeckColumn kind=kind /> }
                />
                <Show when=move || can_add.get()>
                    <div class="deck-add-wrap">
                        <button
                            class="deck-add"
                            on:click=move |_| add_open.update(|v| *v = !*v)
                            aria-label="列を追加"
                            title="列を追加"
                        >
                            <Icon icon=id::FiPlus width="20" height="20" />
                        </button>
                        <Show when=move || add_open.get()>
                            <div class="wf-menu-scrim" on:click=move |_| add_open.set(false) />
                            <div class="wf-pop deck-add-pop" role="menu">
                                {move || missing.get().into_iter().map(|kind| {
                                    view! {
                                        <button
                                            class="wf-pop-item"
                                            on:click=move |_| {
                                                deck.add(kind);
                                                add_open.set(false);
                                            }
                                        >
                                            {kind.label()}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </Show>
                    </div>
                </Show>
            </div>
        </Shell>
    }
}

#[component]
fn DeckColumn(kind: ColumnKind) -> impl IntoView {
    let id = format!("deck-col-{}", kind.id());
    view! {
        <section class=format!("deck-col deck-col-{}", kind.id()) id=id data-path=kind.path()>
            <ColumnHeader kind=kind />
            <div class="deck-col-body">
                {match kind {
                    ColumnKind::Notifications => view! { <NotificationsColumn /> }.into_any(),
                    _ => view! { <TimelineColumn kind=kind /> }.into_any(),
                }}
            </div>
        </section>
    }
}

#[component]
fn ColumnHeader(kind: ColumnKind) -> impl IntoView {
    let deck = expect_context::<DeckStore>();
    let menu = RwSignal::new(false);
    let icon = match kind {
        ColumnKind::Home => id::FiHome,
        ColumnKind::Local => id::FiUsers,
        ColumnKind::Global => id::FiGlobe,
        ColumnKind::Notifications => id::FiBell,
    };
    let can_remove = Signal::derive(move || deck.columns.get().len() > 1);
    let idx = Signal::derive(move || {
        deck.columns
            .get()
            .iter()
            .position(|c| *c == kind)
            .unwrap_or(0)
    });
    let last = Signal::derive(move || {
        let cols = deck.columns.get();
        cols.last().copied() == Some(kind) || !cols.contains(&kind)
    });

    view! {
        <header class="deck-col-head">
            <span class=format!("deck-mark deck-mark-{}", kind.id()) aria-hidden="true">
                <Icon icon=icon width="18" height="18" />
            </span>
            <div class="deck-col-title">
                <h2>{kind.label()}</h2>
                <span class="deck-live">"live"</span>
            </div>
            <div class="wf-ico-wrap">
                <button
                    class="wf-ico-btn"
                    on:click=move |_| menu.update(|v| *v = !*v)
                    aria-label="列の操作"
                    title="列の操作"
                >
                    <Icon icon=id::FiMoreHorizontal width="16" height="16" />
                </button>
                <Show when=move || menu.get()>
                    <div class="wf-menu-scrim" on:click=move |_| menu.set(false) />
                    <div class="wf-pop deck-col-pop" role="menu">
                        <button
                            class="wf-pop-item"
                            disabled=move || idx.get() == 0
                            on:click=move |_| {
                                deck.move_left(kind);
                                menu.set(false);
                            }
                        >
                            "左へ"
                        </button>
                        <button
                            class="wf-pop-item"
                            disabled=move || last.get()
                            on:click=move |_| {
                                deck.move_right(kind);
                                menu.set(false);
                            }
                        >
                            "右へ"
                        </button>
                        <button
                            class="wf-pop-item danger"
                            disabled=move || !can_remove.get()
                            on:click=move |_| {
                                deck.remove(kind);
                                menu.set(false);
                            }
                        >
                            "列を外す"
                        </button>
                        {move || {
                            let extra = deck.missing();
                            extra.into_iter().map(|add_kind| {
                                view! {
                                    <button
                                        class="wf-pop-item"
                                        on:click=move |_| {
                                            deck.add(add_kind);
                                            menu.set(false);
                                        }
                                    >
                                        {format!("{} を追加", add_kind.label())}
                                    </button>
                                }
                            }).collect_view()
                        }}
                    </div>
                </Show>
            </div>
        </header>
    }
}

#[component]
fn TimelineColumn(kind: ColumnKind) -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let stream = expect_context::<StreamStore>();
    let notes = RwSignal::<Vec<Note>>::new(vec![]);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);
    let kind_str = kind.api_kind().unwrap_or("home");

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
            ColumnKind::Local => note.author.host.is_none(),
            ColumnKind::Global => true,
            ColumnKind::Home => {
                me_id.as_deref() == Some(note.author.id.as_str())
                    || notes
                        .with_untracked(|items| items.iter().any(|n| n.author.id == note.author.id))
            }
            ColumnKind::Notifications => false,
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
    }
}

#[component]
fn NotificationsColumn() -> impl IntoView {
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
