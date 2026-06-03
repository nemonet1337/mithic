use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

use super::avatar::{Avatar, AvatarAccent, AvatarSize};
use crate::store::{ComposeStore, NotificationStore};

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
                {nav_items.into_iter().map(|(item_id, label, num, href)| {
                    let on = active == item_id;
                    let badge = match item_id {
                        "notif" => notifications.unread_notifications,
                        "dm"    => notifications.unread_messages,
                        _       => RwSignal::new(0),
                    };
                    view! {
                        <A href=href attr:class=move || if on { "wf-spine-item on" } else { "wf-spine-item" }>
                            <span class="wf-spine-num">{num}</span>
                            <span class="wf-spine-icon-inline">{nav_icon(item_id)}</span>
                            <span class="wf-spine-label">{label}</span>
                            <Show when=move || (badge.get() > 0)>
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
            <A href="/">{nav_icon("home")}</A>
            <A href="/search">{nav_icon("search")}</A>
            <button on:click=move |_| compose.open()>"＋"</button>
            <A href="/notifications">{nav_icon("notif")}</A>
            <A href="/dm">{nav_icon("dm")}</A>
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

fn nav_icon(name: &str) -> AnyView {
    let icon = match name {
        "home" => id::FiHome,
        "search" => id::FiSearch,
        "notif" => id::FiBell,
        "dm" => id::FiMail,
        "profile" => id::FiUser,
        "settings" => id::FiSettings,
        _ => id::FiCircle,
    };
    view! { <Icon icon=icon width="16" height="16" /> }.into_any()
}
