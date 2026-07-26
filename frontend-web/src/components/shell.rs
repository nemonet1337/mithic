use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

use super::avatar::{Avatar, AvatarAccent, AvatarSize};
use crate::store::{AuthStore, ComposeStore, NotificationStore};

// ===========================================================
// Shell — spine layout (paper & ink)
// ===========================================================
#[component]
pub fn Shell(
    #[prop(into)] active: String,
    #[prop(default = true)] right_rail: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="app-root wf-shell" id="app-root">
            <Sidebar active=active.clone() />
            <main class="wf-main">
                {children()}
            </main>
            <Show when=move || right_rail>
                <RightRail />
            </Show>

            // モバイル: トップバー + ボトムタブ + FAB
            <MobileTopBar active=active.clone() />
            <MobileBottomNav active=active.clone() />
            <MobileFab />
        </div>
    }
}

// ===========================================================
// Sidebar (spine)
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
            ("01", "home", "ホーム", id::FiHome, "/".to_string()),
            ("02", "search", "検索", id::FiSearch, "/search".to_string()),
            ("03", "notif", "通知", id::FiBell, "/notifications".to_string()),
            ("04", "dm", "メッセージ", id::FiMail, "/dm".to_string()),
            ("05", "profile", "プロフィール", id::FiUser, p_href),
            ("06", "settings", "設定", id::FiSettings, "/settings".to_string()),
        ]
    };

    view! {
        <aside class="wf-spine">
            // ブランドロゴ (stamp)
            <A href="/" attr:class="wf-spine-head">
                <span class="wf-stamp">"m"</span>
                <span class="wf-mark wf-mark-md">"[m]"<span class="br">"mithic"</span></span>
            </A>
            <hr class="wf-spine-rule" />

            // ナビ (番号付き spine)
            <nav class="wf-spine-nav">
                {move || nav_items().into_iter().map(|(num, item_id, label, icon, href)| {
                    let is_active = active == item_id;
                    let badge = match item_id {
                        "notif" => notifications.unread_notifications,
                        "dm"    => notifications.unread_messages,
                        _       => RwSignal::new(0),
                    };
                    view! {
                        <A href=href attr:class=move || {
                            if is_active { "wf-spine-item active" } else { "wf-spine-item" }
                        }>
                            <span class="wf-spine-num">{num}</span>
                            <Icon icon=icon width="18" height="18" />
                            <span class="flex-1">{label}</span>
                            <Show when=move || (badge.get() > 0u32)>
                                <span class="wf-badge">{move || badge.get().to_string()}</span>
                            </Show>
                        </A>
                    }
                }).collect_view()}
            </nav>

            // 投稿ボタン (stamp btn)
            <button class="wf-stamp-btn" on:click=move |_| compose.open()>
                <Icon icon=id::FiEdit width="16" height="16" />
                "NEW + 投稿"
            </button>

            // ユーザーフッター
            <div class="wf-spine-foot">
                {move || me.get().map(|u| {
                    let href = format!("/profile/{}", u.username);
                    view! {
                        <A href=href attr:class="flex items-center gap-2 min-w-0 flex-1">
                            <Avatar user=u.clone() size=AvatarSize::Sm accent=AvatarAccent::None />
                            <div class="min-w-0">
                                <div class="wf-foot-name">{u.name()}</div>
                                <div class="wf-foot-sig">"SIG·ok"</div>
                            </div>
                        </A>
                    }
                })}
            </div>
        </aside>
    }
}

// ===========================================================
// モバイル トップバー
// ===========================================================
#[component]
pub fn MobileTopBar(#[prop(into)] active: String) -> impl IntoView {
    let titles = [
        ("home", "ホーム"),
        ("search", "検索"),
        ("notif", "通知"),
        ("dm", "メッセージ"),
        ("profile", "プロフィール"),
        ("settings", "設定"),
    ];
    let title = Signal::derive(move || {
        titles
            .iter()
            .find(|(id, _)| *id == active)
            .map(|(_, t)| t.to_string())
            .unwrap_or_else(|| "mithic".to_string())
    });
    view! {
        <header class="an-top">
            <span class="wf-mark wf-mark-sm">"[m]"</span>
            <span class="an-folio">"[ 01 ]"</span>
            <span class="an-title">{move || title.get()}</span>
        </header>
    }
}

// ===========================================================
// モバイル FAB 投稿
// ===========================================================
#[component]
fn MobileFab() -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    view! {
        <button class="an-fab" on:click=move |_| compose.open() aria-label="投稿">
            <Icon icon=id::FiEdit width="24" height="24" />
        </button>
    }
}

// ===========================================================
// モバイル ボトムタブ
// ===========================================================
#[component]
pub fn MobileBottomNav(#[prop(into)] active: String) -> impl IntoView {
    let notifications = expect_context::<NotificationStore>();
    let act_home = active.clone();
    let act_search = active.clone();
    let act_notif = active.clone();
    let act_dm = active.clone();
    let act_profile = active.clone();

    view! {
        <nav class="an-tab">
            <A href="/" attr:class=move || if act_home == "home" { "active" } else { "" }>
                <Icon icon=id::FiHome width="20" height="20" />
                "01"
            </A>
            <A href="/search" attr:class=move || if act_search == "search" { "active" } else { "" }>
                <Icon icon=id::FiSearch width="20" height="20" />
                "02"
            </A>
            <A href="/notifications" attr:class=move || if act_notif == "notif" { "active" } else { "" }>
                <span class="relative inline-flex">
                    <Icon icon=id::FiBell width="20" height="20" />
                    <Show when=move || (notifications.unread_notifications.get() > 0u32)>
                        <span class="wf-badge" style="position:absolute;top:-6px;right:-8px;margin:0" />
                    </Show>
                </span>
                "03"
            </A>
            <A href="/dm" attr:class=move || if act_dm == "dm" { "active" } else { "" }>
                <span class="relative inline-flex">
                    <Icon icon=id::FiMail width="20" height="20" />
                    <Show when=move || (notifications.unread_messages.get() > 0u32)>
                        <span class="wf-badge" style="position:absolute;top:-6px;right:-8px;margin:0" />
                    </Show>
                </span>
                "04"
            </A>
            <A href="/settings" attr:class=move || if act_profile == "profile" || act_profile == "settings" { "active" } else { "" }>
                <Icon icon=id::FiSettings width="20" height="20" />
                "06"
            </A>
        </nav>
    }
}

// ===========================================================
// TopBar (folio + hand title + seg tabs)
// ===========================================================
#[component]
pub fn TopBar(
    #[prop(into)] title: String,
    #[prop(into, optional)] folio: Option<String>,
    #[prop(optional)] tabs: Option<Vec<(&'static str, &'static str, bool)>>,
) -> impl IntoView {
    let tabs = tabs.unwrap_or_default();
    let has_tabs = !tabs.is_empty();
    let navigate = leptos_router::hooks::use_navigate();

    view! {
        <header class="wf-topbar">
            <div class="flex items-center gap-3">
                {folio.map(|f| view! { <span class="wf-folio">{f}</span> })}
                <h1 class="wf-title">{title}</h1>
            </div>
            <Show when=move || has_tabs>
                <div class="wf-seg">
                    {tabs
                        .iter()
                        .copied()
                        .map(|(label, href, active)| {
                            let nav = navigate.clone();
                            view! {
                                <span
                                    class=move || if active { "wf-seg-item active" } else { "wf-seg-item" }
                                    on:click=move |_| nav(href, Default::default())
                                    role="button"
                                >
                                    <span class="wf-seg-num">""</span>
                                    {label}
                                </span>
                            }
                        })
                        .collect_view()}
                </div>
            </Show>
        </header>
    }
}

// ===========================================================
// RightRail (marginalia)
// ===========================================================
#[component]
pub fn RightRail() -> impl IntoView {
    let trends = vec![
        ("#design", "2.1k"),
        ("#typography", "1.8k"),
        ("#ux", "1.4k"),
        ("#wasm", "980"),
        ("#federated", "760"),
    ];
    let suggested = vec!["@inkwell", "@paperpress", "@handdrawn", "@signal", "@noise"];

    view! {
        <aside class="wf-rail">
            <div class="wf-find">
                <Icon icon=id::FiSearch width="16" height="16" attr:class="opacity-40" />
                <input type="text" placeholder="find…" />
            </div>

            <div class="wf-rail-card">
                <div class="wf-rail-head">
                    <span class="wf-rail-tag">"[ TRENDING ]"</span>
                    <span class="wf-rail-jp">"急上昇"</span>
                </div>
                {trends.into_iter().enumerate().map(|(i, (tag, count))| view! {
                    <A href=format!("/search?tag={}", tag.trim_start_matches('#')) attr:class="wf-rail-row">
                        <span class="flex items-center">
                            <span class="wf-rail-rank">{format!("{:02}.", i + 1)}</span>
                            <span class="wf-rail-name">{tag}</span>
                        </span>
                        <span class="wf-rail-meta">{count}</span>
                    </A>
                }).collect_view()}
            </div>

            <div class="wf-rail-card">
                <div class="wf-rail-head">
                    <span class="wf-rail-tag">"[ SUGGESTED ]"</span>
                    <span class="wf-rail-jp">"おすすめ"</span>
                </div>
                {suggested.into_iter().enumerate().map(|(i, handle)| view! {
                    <div class="wf-rail-row">
                        <span class="flex items-center">
                            <span class="wf-rail-rank">{format!("{:02}.", i + 1)}</span>
                            <span class="wf-rail-name">{handle}</span>
                        </span>
                        <button class="wf-follow-pill">"追う"</button>
                    </div>
                }).collect_view()}
            </div>

            <div class="wf-rail-foot">"— mithic · signal not noise —"</div>
        </aside>
    }
}

// ===========================================================
// BottomNav (後方互換エイリアス、未使用)
// ===========================================================
#[component]
pub fn BottomNav() -> impl IntoView {
    view! { <div /> }
}
