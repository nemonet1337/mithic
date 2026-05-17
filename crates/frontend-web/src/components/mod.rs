use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::models::{Note, NoteVisibility, User};
use crate::store::{AuthStore, ComposeStore, NotificationStore};

#[component]
pub fn Protected(children: Children) -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let auth_for_redirect = auth.clone();
    let navigate = use_navigate();
    Effect::new(move |_| {
        if !auth_for_redirect.is_authenticated() {
            navigate("/login", Default::default());
        }
    });
    if auth.is_authenticated() {
        children().into_any()
    } else {
        view! { <></> }.into_any()
    }
}

#[component]
pub fn Shell(
    #[prop(into)] active: String,
    #[prop(default = true)] right_rail: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="wf app-shell">
            <Sidebar active=active />
            <main class="main-pane">{children()}</main>
            <Show when=move || right_rail>
                <RightRail />
            </Show>
            <BottomNav />
        </div>
    }
}

#[component]
pub fn TopBar(
    #[prop(into)] title: String,
    #[prop(into, optional)] folio: Option<String>,
    #[prop(optional)] tabs: Option<Vec<(&'static str, &'static str)>>,
    #[prop(into, optional)] active_tab: Option<String>,
) -> impl IntoView {
    let tab_view = tabs
        .unwrap_or_default()
        .into_iter()
        .map(|(label, href)| {
            let is_active = active_tab.as_deref() == Some(href);
            view! {
                <A href=href attr:class=move || if is_active { "t on" } else { "t" }>{label}</A>
            }
        })
        .collect_view();

    view! {
        <header class="wf-spread top-bar">
            <div class="wf-row top-bar-title">
                <span class="wf-mono top-folio">{folio.unwrap_or_else(|| "01".into())}</span>
                <span class="wf-hand top-title">{title}</span>
            </div>
            <div class="wf-tabs top-tabs">{tab_view}</div>
        </header>
    }
}

#[component]
pub fn Sidebar(#[prop(into)] active: String) -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    let notifications = expect_context::<NotificationStore>();
    let nav_items = vec![
        ("home", "ホーム", "01", "/"),
        ("search", "検索", "02", "/search"),
        ("notif", "通知", "03", "/notifications"),
        ("dm", "メッセージ", "04", "/dm"),
        ("profile", "プロフィール", "05", "/you"),
        ("settings", "設定", "06", "/settings"),
    ];

    view! {
        <aside class="wf-spine responsive-sidebar">
            <A href="/" attr:class="wf-mark">
                <span class="wf-mark-bracket">"["</span>
                <span class="wf-mark-glyph">"m"</span>
                <span class="wf-mark-bracket">"]"</span>
                <span class="wf-mark-text">"mithic"</span>
            </A>
            <div class="wf-spine-rule" />
            <nav class="wf-stack nav-stack">
                {nav_items.into_iter().map(|(id, label, num, href)| {
                    let on = active == id;
                    let badge = match id {
                        "notif" => notifications.unread_notifications,
                        "dm" => notifications.unread_messages,
                        _ => RwSignal::new(0),
                    };
                    view! {
                        <A href=href attr:class=move || if on { "wf-spine-item on" } else { "wf-spine-item" }>
                            <span class="wf-spine-num">{num}</span>
                            <span class="wf-spine-icon-inline">{nav_symbol(id)}</span>
                            <span class="wf-spine-label">{label}</span>
                            <Show when=move || { badge.get() > 0 }>
                                <span class="nav-badge">{move || badge.get().to_string()}</span>
                            </Show>
                        </A>
                    }
                }).collect_view()}
            </nav>
            <div class="sidebar-spacer" />
            <button class="wf-stamp-btn" on:click=move |_| compose.open()>
                <span class="wf-mono stamp-kicker">"NEW"</span>
                <span class="wf-hand stamp-main">"+ 投稿"</span>
            </button>
            <div class="wf-spine-foot">
                <div class="wf-av sm accent" />
                <div class="wf-col wf-grow">
                    <span class="wf-mono user-mini">"@you"</span>
                    <span class="wf-mono user-sig">"SIG · ok"</span>
                </div>
            </div>
        </aside>
    }
}

#[component]
pub fn BottomNav() -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    view! {
        <nav class="bottom-nav">
            <A href="/">"⌂"</A>
            <A href="/search">"⌕"</A>
            <button on:click=move |_| compose.open()>"＋"</button>
            <A href="/notifications">"◌"</A>
            <A href="/dm">"✉"</A>
        </nav>
    }
}

#[component]
pub fn RightRail() -> impl IntoView {
    let trends = vec!["#design", "#ux", "#typography", "#wireframe"];
    view! {
        <aside class="right-rail wf-col">
            <div class="wf-input dashed"><span class="wf-mono search-symbol">"⌕"</span>"find…"</div>
            <section>
                <div class="wf-label rail-label">"[ 急上昇 / TRENDING ]"</div>
                <div class="wf-stack">
                    {trends.into_iter().enumerate().map(|(index, tag)| view! {
                        <A href=format!("/search?tag={}", tag.trim_start_matches('#')) attr:class="wf-spread rail-row">
                            <span class="wf-row">
                                <span class="wf-mono rail-index">{format!("{:02}.", index + 1)}</span>
                                <span class="wf-tag">{tag}</span>
                            </span>
                            <span class="wf-mono rail-count">{format!("{}.{}k", index + 2, index + 4)}</span>
                        </A>
                    }).collect_view()}
                </div>
            </section>
            <section>
                <div class="wf-label rail-label">"[ おすすめ / SUGGESTED ]"</div>
                <div class="wf-stack suggested-stack">
                    {(1..=3).map(|i| view! {
                        <div class="wf-row suggested-user">
                            <div class="wf-av sm" />
                            <div class="wf-col wf-grow">
                                <span class="wf-hand suggested-name">{format!("User {i}")}</span>
                                <span class="wf-mono suggested-handle">{format!("@user_{i}")}</span>
                            </div>
                            <button class="wf-btn sm">"+ 追う"</button>
                        </div>
                    }).collect_view()}
                </div>
            </section>
            <div class="wf-mono rail-footer">"— mithic · signal not noise —"</div>
        </aside>
    }
}

#[component]
pub fn Avatar(user: User, #[prop(default = AvatarSize::Md)] size: AvatarSize) -> impl IntoView {
    let initials = user.initials();
    let label = user.handle();
    let name = user.name();
    let avatar_url = user.avatar_url.clone();
    let class = format!("wf-av {} avatar-content", size.class_name());
    view! {
        <div class=class aria-label=label>
            {if let Some(url) = avatar_url {
                view! { <img src=url alt=name loading="lazy" /> }.into_any()
            } else {
                view! { <span>{initials}</span> }.into_any()
            }}
        </div>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    Sm,
    Md,
    Lg,
    Xl,
}

impl AvatarSize {
    fn class_name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }
}

#[component]
pub fn PostCard(note: Note, #[prop(default = false)] flat: bool) -> impl IntoView {
    let author = note.author.clone();
    let route = format!("/{}", author.route_handle());
    let note_href = format!("/notes/{}", note.id);
    let has_attachments = !note.attachments.is_empty();
    let card_class = if flat { "wf-entry flat" } else { "wf-entry" };
    let date = if note.created_at.contains('m') {
        "NOW"
    } else {
        "MAY·10"
    };
    view! {
        <article class=card_class>
            <div class="wf-stamp accent">
                <span class="wf-stamp-date">{date}</span>
                <span class="wf-stamp-time">{note.created_at.clone()}</span>
            </div>
            <div class="wf-grow post-main">
                <div class="wf-spread post-header">
                    <A href=route attr:class="wf-row post-author">
                        <Avatar user=author.clone() size=AvatarSize::Sm />
                        <span class="wf-hand post-name">{author.name()}</span>
                        <span class="wf-mono post-handle">{author.handle()}</span>
                    </A>
                    <A href=note_href attr:class="wf-mono post-menu">"···"</A>
                </div>
                <PostBody content=note.content.clone() cw=note.cw.clone() />
                <Show when=move || has_attachments>
                    <div class="wf-media post-media">"media"</div>
                </Show>
                <PostActions note=note />
            </div>
        </article>
    }
}

#[component]
pub fn PostBody(content: String, cw: Option<String>) -> impl IntoView {
    let has_cw = cw.is_some();
    let cw_text = cw.unwrap_or_default();
    let expanded = RwSignal::new(!has_cw);
    view! {
        <div class="post-body">
            {if has_cw {
                view! {
                    <div class="wf-card dashed cw-box">
                        <span class="wf-label">"CW"</span>
                        <strong>{cw_text}</strong>
                        <button class="wf-btn sm ghost" on:click=move |_| expanded.update(|value| *value = !*value)>
                            {move || if expanded.get() { "隠す" } else { "開く" }}
                        </button>
                    </div>
                }.into_any()
            } else {
                view! {}.into_any()
            }}
            <Show when=move || expanded.get()>
                <MfmText text=content.clone() />
            </Show>
        </div>
    }
}

#[component]
pub fn PostActions(note: Note) -> impl IntoView {
    view! {
        <div class="wf-row post-actions">
            <button class="wf-btn sm">"↩ 返信"</button>
            <button class="wf-btn sm">{format!("↻ {}", note.renote_count)}</button>
            <button class="wf-btn sm">"＋ REACT"</button>
            {note.reactions.into_iter().map(|reaction| view! {
                <span class=move || if reaction.reacted_by_me { "wf-pill accent" } else { "wf-pill" }>
                    {reaction.emoji} " " {reaction.count.to_string()}
                </span>
            }).collect_view()}
            <span class="wf-mono action-summary">{format!("↪ {}", note.quote_count)}</span>
        </div>
    }
}

#[component]
pub fn MfmText(text: String) -> impl IntoView {
    view! {
        <span class="mfm-text">
            {shared::mfm::parse(&text).into_iter().map(render_mfm_node).collect_view()}
        </span>
    }
}

fn render_mfm_node(node: shared::mfm::MfmNode) -> AnyView {
    match node {
        shared::mfm::MfmNode::Text(text) => view! { <span>{text}</span> }.into_any(),
        shared::mfm::MfmNode::Mention(acct) => view! {
            <A href=format!("/{}", acct) attr:class="text-accent">"@"{acct}</A>
        }
        .into_any(),
        shared::mfm::MfmNode::Hashtag(tag) => view! {
            <A href=format!("/search?tag={tag}") attr:class="text-accent">"#"{tag}</A>
        }
        .into_any(),
        shared::mfm::MfmNode::Url(url) => {
            let href = url.clone();
            view! {
                <a class="text-accent break-all" href=href target="_blank" rel="noreferrer">{url}</a>
            }
            .into_any()
        }
        shared::mfm::MfmNode::Bold(children) => view! {
            <strong>{children.into_iter().map(render_mfm_node).collect_view()}</strong>
        }
        .into_any(),
        shared::mfm::MfmNode::Italic(children) => view! {
            <em>{children.into_iter().map(render_mfm_node).collect_view()}</em>
        }
        .into_any(),
        shared::mfm::MfmNode::Emoji(name) => {
            view! { <span class="inline-emoji">{format!(":{name}:")}</span> }.into_any()
        }
        shared::mfm::MfmNode::InlineCode(code) => view! { <code>{code}</code> }.into_any(),
        shared::mfm::MfmNode::LineBreak => view! { <br /> }.into_any(),
    }
}

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

fn nav_symbol(id: &str) -> &'static str {
    match id {
        "home" => "⌂",
        "search" => "⌕",
        "notif" => "◌",
        "dm" => "✉",
        "profile" => "◎",
        "settings" => "⚙",
        _ => "•",
    }
}
