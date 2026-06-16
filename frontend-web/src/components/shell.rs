use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

use super::avatar::{Avatar, AvatarAccent, AvatarSize};
use crate::store::{AuthStore, ComposeStore, NotificationStore};

// ===========================================================
// Shell — メインレイアウト (3カラム / レスポンシブ drawer)
// ===========================================================
#[component]
pub fn Shell(
    #[prop(into)] active: String,
    #[prop(default = true)] right_rail: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="app-root" id="app-root">
            // モバイル用 drawer (DaisyUI)
            <div class="drawer lg:drawer-open h-dvh">
                <input id="sidebar-drawer" type="checkbox" class="drawer-toggle" />

                // メインコンテンツ
                <div class="drawer-content flex flex-col h-dvh overflow-hidden">
                    // モバイル Navbar
                    <div class="navbar bg-base-100/80 backdrop-blur-md border-b border-base-300 lg:hidden sticky top-0 z-10 min-h-[52px]">
                        <div class="navbar-start">
                            <label for="sidebar-drawer" class="btn btn-ghost btn-sm btn-square">
                                <Icon icon=id::FiMenu width="20" height="20" />
                            </label>
                        </div>
                        <div class="navbar-center">
                            <a href="/" class="font-black text-xl text-primary tracking-tighter">"[m]"</a>
                        </div>
                        <div class="navbar-end">
                            <ComposeButton />
                        </div>
                    </div>

                    // メインペイン + 右レール
                    <div class="flex flex-1 overflow-hidden">
                        <main class="flex flex-col flex-1 overflow-hidden border-x border-base-300">
                            {children()}
                        </main>
                        <Show when=move || right_rail>
                            <RightRail />
                        </Show>
                    </div>

                    // モバイルボトムナビ
                    <MobileBottomNav active=active.clone() />
                </div>

                // Drawer サイドバー
                <div class="drawer-side z-20">
                    <label for="sidebar-drawer" aria-label="close sidebar" class="drawer-overlay"></label>
                    <Sidebar active=active />
                </div>
            </div>
        </div>
    }
}

// ===========================================================
// Sidebar
// ===========================================================
#[component]
pub fn Sidebar(#[prop(into)] active: String) -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    let notifications = expect_context::<NotificationStore>();
    let auth = expect_context::<AuthStore>();
    let me = auth.me;

    let profile_href = move || {
        if let Some(user) = me.get() {
            format!("/profile/{}", user.username)
        } else {
            "/you".into()
        }
    };

    let nav_items = move || {
        let p_href = profile_href();
        vec![
            ("home", "ホーム", id::FiHome, "/".to_string()),
            ("search", "検索", id::FiSearch, "/search".to_string()),
            ("notif", "通知", id::FiBell, "/notifications".to_string()),
            ("dm", "DM", id::FiMail, "/dm".to_string()),
            ("profile", "プロフィール", id::FiUser, p_href),
            ("settings", "設定", id::FiSettings, "/settings".to_string()),
        ]
    };

    view! {
        <aside class="responsive-sidebar w-64 bg-base-100 flex flex-col h-dvh border-r border-base-300">
            <div class="sidebar-inner">
                // ブランドロゴ
                <A href="/" attr:class="brand-logo group">
                    <span class="brand-glyph group-hover:text-primary transition-colors">"[m]"</span>
                    <span class="brand-name">"mithic"</span>
                </A>

                // ナビゲーション
                <nav class="flex flex-col gap-1 flex-1">
                    {move || nav_items().into_iter().map(|(item_id, label, icon, href)| {
                        let is_active = active == item_id;
                        let badge = match item_id {
                            "notif" => notifications.unread_notifications,
                            "dm"    => notifications.unread_messages,
                            _       => RwSignal::new(0),
                        };
                        view! {
                            <A href=href attr:class=move || {
                                if is_active { "nav-item active" } else { "nav-item" }
                            }>
                                <span class="relative">
                                    <Icon icon=icon width="20" height="20" />
                                    <Show when=move || (badge.get() > 0)>
                                        <span class="badge badge-primary badge-xs absolute -top-1 -right-1" />
                                    </Show>
                                </span>
                                <span class="flex-1">{label}</span>
                                <Show when=move || (badge.get() > 0)>
                                    <span class="badge badge-primary badge-sm font-mono">
                                        {move || badge.get().to_string()}
                                    </span>
                                </Show>
                            </A>
                        }
                    }).collect_view()}
                </nav>

                // 投稿ボタン (デスクトップ)
                <div class="hidden lg:block mt-2">
                    <button
                        class="btn btn-primary w-full rounded-2xl font-bold text-base"
                        on:click=move |_| compose.open()
                    >
                        <Icon icon=id::FiEdit width="18" height="18" />
                        "投稿する"
                    </button>
                </div>

                // ユーザーフッター
                <div class="mt-auto pt-4 border-t border-base-300">
                    {move || me.get().map(|u| {
                        let href = format!("/profile/{}", u.username);
                        view! {
                            <A href=href attr:class="flex items-center gap-3 p-2 rounded-xl hover:bg-base-200 transition-colors">
                                <Avatar user=u.clone() size=AvatarSize::Sm accent=AvatarAccent::None />
                                <div class="flex flex-col min-w-0 flex-1">
                                    <span class="font-semibold text-sm truncate">{u.name()}</span>
                                    <span class="text-xs font-mono opacity-50 truncate">{format!("@{}", u.username)}</span>
                                </div>
                                <Icon icon=id::FiMoreHorizontal width="16" height="16" attr:class="opacity-40" />
                            </A>
                        }
                    })}
                </div>
            </div>
        </aside>
    }
}

// ===========================================================
// モバイル用浮遊投稿ボタン
// ===========================================================
#[component]
fn ComposeButton() -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    view! {
        <button
            class="btn btn-primary btn-sm btn-circle"
            on:click=move |_| compose.open()
        >
            <Icon icon=id::FiEdit width="16" height="16" />
        </button>
    }
}

// ===========================================================
// モバイルボトムナビ
// ===========================================================
#[component]
pub fn MobileBottomNav(#[prop(into)] active: String) -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    let notifications = expect_context::<NotificationStore>();

    let act_home = active.clone();
    let act_search = active.clone();
    let act_notif = active.clone();
    let act_dm = active.clone();

    view! {
        <div class="btm-nav btm-nav-custom lg:hidden z-10">
            <A href="/" attr:class=move || {
                if act_home == "home" { "active text-primary" } else { "" }
            }>
                <Icon icon=id::FiHome width="22" height="22" />
            </A>
            <A href="/search" attr:class=move || {
                if act_search == "search" { "active text-primary" } else { "" }
            }>
                <Icon icon=id::FiSearch width="22" height="22" />
            </A>
            <button class="btn btn-primary btn-circle btn-sm" on:click=move |_| compose.open()>
                <Icon icon=id::FiPlusCircle width="22" height="22" />
            </button>
            <A href="/notifications" attr:class=move || {
                let extra = if act_notif == "notif" { "active text-primary" } else { "" };
                extra.to_string()
            }>
                <div class="indicator">
                    <Icon icon=id::FiBell width="22" height="22" />
                    <Show when=move || (notifications.unread_notifications.get() > 0)>
                        <span class="badge badge-primary badge-xs indicator-item" />
                    </Show>
                </div>
            </A>
            <A href="/dm" attr:class=move || {
                if act_dm == "dm" { "active text-primary" } else { "" }
            }>
                <div class="indicator">
                    <Icon icon=id::FiMail width="22" height="22" />
                    <Show when=move || (notifications.unread_messages.get() > 0)>
                        <span class="badge badge-primary badge-xs indicator-item" />
                    </Show>
                </div>
            </A>
        </div>
    }
}

// ===========================================================
// TopBar (各ページ上部)
// ===========================================================
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
                <A href=href attr:class=move || {
                    if is_active {
                        "tab tab-active font-semibold"
                    } else {
                        "tab"
                    }
                }>
                    {label}
                </A>
            }
        })
        .collect_view();

    view! {
        <header class="top-bar px-4 flex flex-col justify-center">
            <div class="flex items-center gap-3">
                {folio.map(|f| view! {
                    <span class="font-mono text-xs opacity-30 select-none">{f}</span>
                })}
                <h1 class="font-bold text-base">{title}</h1>
            </div>
            <div class="tabs tabs-bordered -mb-px">
                {tab_view}
            </div>
        </header>
    }
}

// ===========================================================
// RightRail
// ===========================================================
#[component]
pub fn RightRail() -> impl IntoView {
    let trends = vec!["#design", "#ux", "#typography", "#wasm", "#federated"];
    view! {
        <aside class="right-rail w-[300px] hidden xl:flex flex-col">
            // 検索ボックス
            <div class="p-3">
                <div class="input input-bordered flex items-center gap-2 rounded-full">
                    <Icon icon=id::FiSearch width="16" height="16" attr:class="opacity-40" />
                    <input type="text" placeholder="検索…" class="grow bg-transparent outline-none text-sm" />
                </div>
            </div>

            // トレンド
            <div class="card card-compact mx-3 bg-base-200 border border-base-300">
                <div class="card-body gap-3">
                    <h3 class="font-bold text-sm flex items-center gap-2">
                        <span class="font-mono text-xs opacity-40">"[TREND]"</span>
                        "急上昇"
                    </h3>
                    <div class="flex flex-col gap-2">
                        {trends.into_iter().enumerate().map(|(i, tag)| view! {
                            <A
                                href=format!("/search?tag={}", tag.trim_start_matches('#'))
                                attr:class="flex justify-between items-center group py-1"
                            >
                                <div class="flex items-center gap-2">
                                    <span class="font-mono text-xs opacity-30">{format!("{:02}.", i + 1)}</span>
                                    <span class="font-mono text-sm text-primary group-hover:underline">{tag}</span>
                                </div>
                                <span class="font-mono text-xs opacity-40">{format!("{}.{}k", i + 2, i + 4)}</span>
                            </A>
                        }).collect_view()}
                    </div>
                </div>
            </div>

            // フッター
            <div class="px-4 mt-auto pb-4">
                <p class="font-mono text-[10px] opacity-25 text-center">"— mithic · signal not noise —"</p>
            </div>
        </aside>
    }
}

// ===========================================================
// BottomNav (後方互換のためのエイリアス、未使用)
// ===========================================================
#[component]
pub fn BottomNav() -> impl IntoView {
    view! { <div /> }
}
