use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use super::avatar::{Avatar, AvatarSize};
use crate::store::{AuthStore, ComposeStore, NotificationStore};
use shared::User;

fn profile_path(me: Option<&User>) -> String {
    me.map(|u| format!("/profile/{}", u.username))
        .unwrap_or_else(|| "/login".into())
}

fn badge_label(n: u32) -> String {
    if n > 99 { "99+".into() } else { n.to_string() }
}

fn side_class(active: &str, id: &str) -> &'static str {
    if active == id {
        "wf-side-link active"
    } else {
        "wf-side-link"
    }
}

fn clear_session(auth: &AuthStore) {
    let token = auth.token.get_untracked();
    auth.logout();
    if let Some(tok) = token {
        wasm_bindgen_futures::spawn_local(async move {
            let _ = crate::api::auth::logout(&tok).await;
        });
    }
}

#[component]
pub fn Shell(#[prop(into)] active: String, children: Children) -> impl IntoView {
    view! {
        <div class="app-root wf-shell">
            <SideNav active=active.clone() />
            <div class="wf-shell-main">
                <TopNav />
                <main class="wf-main">
                    <div class="wf-main-inner">{children()}</div>
                </main>
            </div>
            <MobileDock active=active />
        </div>
    }
}

#[component]
fn SideNav(#[prop(into)] active: String) -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    let notifications = expect_context::<NotificationStore>();
    let auth = expect_context::<AuthStore>();
    let me = auth.me;
    let navigate = use_navigate();
    let a_home = active.clone();
    let a_local = active.clone();
    let a_global = active.clone();
    let a_notif = active.clone();
    let a_search = active.clone();
    let a_drive = active.clone();

    view! {
        <aside class="wf-side" aria-label="メインナビ">
            <A href="/" attr:class="wf-brand" attr:aria-label="ホーム">
                <span class="wf-brand-mark">"m"</span>
                <span class="wf-brand-name">"mithic"</span>
            </A>
            <nav class="wf-side-nav">
                <A href="/" attr:class=move || side_class(&a_home, "home")>
                    <Icon icon=id::FiHome width="18" height="18" />
                    "ホーム"
                </A>
                <A href="/local" attr:class=move || side_class(&a_local, "local")>
                    <Icon icon=id::FiUsers width="18" height="18" />
                    "ローカル"
                </A>
                <A href="/global" attr:class=move || side_class(&a_global, "global")>
                    <Icon icon=id::FiGlobe width="18" height="18" />
                    "グローバル"
                </A>
                <A href="/notifications" attr:class=move || side_class(&a_notif, "notif")>
                    <Icon icon=id::FiBell width="18" height="18" />
                    "通知"
                    <Show when=move || { notifications.unread_notifications.get() > 0 }>
                        <span class="wf-badge">{move || badge_label(notifications.unread_notifications.get())}</span>
                    </Show>
                </A>
                <A href="/search" attr:class=move || side_class(&a_search, "search")>
                    <Icon icon=id::FiSearch width="18" height="18" />
                    "検索"
                </A>
                <A href="/drive" attr:class=move || side_class(&a_drive, "drive")>
                    <Icon icon=id::FiFolder width="18" height="18" />
                    "ドライブ"
                </A>
            </nav>
            <button class="wf-stamp-btn" on:click=move |_| compose.open()>"投稿"</button>
            <div class="wf-side-account">
                {move || me.get().map(|u| {
                    let name = u.name();
                    let handle = u.handle();
                    view! {
                        <div class="wf-account-head">
                            <div class="wf-account-name">{name}</div>
                            <div class="wf-account-handle">{handle}</div>
                        </div>
                    }
                })}
                {move || {
                    let href = profile_path(me.get().as_ref());
                    view! {
                        <A href=href attr:class="wf-side-link">
                            <Icon icon=id::FiUser width="16" height="16" />
                            "プロフィール"
                        </A>
                    }
                }}
                <A href="/settings" attr:class="wf-side-link">
                    <Icon icon=id::FiSettings width="16" height="16" />
                    "設定"
                </A>
                <button
                    class="wf-side-link wf-side-logout"
                    on:click={
                        let auth = auth.clone();
                        let navigate = navigate.clone();
                        move |_| {
                            clear_session(&auth);
                            navigate("/login", Default::default());
                        }
                    }
                >
                    <Icon icon=id::FiLogOut width="16" height="16" />
                    "ログアウト"
                </button>
            </div>
        </aside>
    }
}

#[component]
fn TopNav() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let me = auth.me;
    let navigate = use_navigate();
    let account_open = RwSignal::new(false);

    view! {
        <header class="wf-topnav">
            <A href="/" attr:class="wf-brand" attr:aria-label="ホーム">
                <span class="wf-brand-mark">"m"</span>
                <span class="wf-brand-name">"mithic"</span>
            </A>
            <div class="wf-topnav-actions">
                <div class="wf-ico-wrap">
                    <button
                        class="wf-ico-btn wf-ico-avatar"
                        on:click=move |_| account_open.update(|v| *v = !*v)
                        aria-label="アカウント"
                        aria-expanded=move || account_open.get().to_string()
                    >
                        {move || me.get().map(|u| view! { <Avatar user=u size=AvatarSize::Sm /> })}
                    </button>
                    <Show when=move || account_open.get()>
                        <div class="wf-menu-scrim" on:click=move |_| account_open.set(false) />
                        <div class="wf-pop wf-nav-pop wf-nav-pop-end" role="menu">
                            {move || {
                                me.get().map(|u| {
                                    let name = u.name();
                                    let handle = u.handle();
                                    view! {
                                        <div class="wf-account-head">
                                            <div class="wf-account-name">{name}</div>
                                            <div class="wf-account-handle">{handle}</div>
                                        </div>
                                    }
                                })
                            }}
                            {move || {
                                let href = profile_path(me.get().as_ref());
                                view! {
                                    <A href=href attr:class="wf-pop-item" on:click=move |_| account_open.set(false)>
                                        <Icon icon=id::FiUser width="16" height="16" />
                                        "プロフィール"
                                    </A>
                                }
                            }}
                            <A href="/settings" attr:class="wf-pop-item" on:click=move |_| account_open.set(false)>
                                <Icon icon=id::FiSettings width="16" height="16" />
                                "設定"
                            </A>
                            <A href="/drive" attr:class="wf-pop-item" on:click=move |_| account_open.set(false)>
                                <Icon icon=id::FiFolder width="16" height="16" />
                                "ドライブ"
                            </A>
                            <hr class="wf-rule" />
                            <button
                                class="wf-pop-item danger"
                                on:click={
                                    let auth = auth.clone();
                                    let navigate = navigate.clone();
                                    move |_| {
                                        account_open.set(false);
                                        clear_session(&auth);
                                        navigate("/login", Default::default());
                                    }
                                }
                            >
                                <Icon icon=id::FiLogOut width="16" height="16" />
                                "ログアウト"
                            </button>
                        </div>
                    </Show>
                </div>
            </div>
        </header>
    }
}

#[component]
fn MobileDock(#[prop(into)] active: String) -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    let notifications = expect_context::<NotificationStore>();
    let auth = expect_context::<AuthStore>();
    let me = auth.me;
    let more_open = RwSignal::new(false);

    let act_home = active.clone();
    let act_search = active.clone();
    let act_notif = active.clone();
    let act_more = active.clone();

    view! {
        <nav class="wf-dock" aria-label="メインナビ">
            <A
                href="/"
                attr:class=move || {
                    if act_home == "home" { "wf-dock-item active" } else { "wf-dock-item" }
                }
                attr:aria-label="ホーム"
            >
                <Icon icon=id::FiHome width="20" height="20" />
            </A>
            <A
                href="/search"
                attr:class=move || {
                    if act_search == "search" { "wf-dock-item active" } else { "wf-dock-item" }
                }
                attr:aria-label="検索"
            >
                <Icon icon=id::FiSearch width="20" height="20" />
            </A>
            <button class="wf-dock-compose" on:click=move |_| compose.open() aria-label="投稿">
                <Icon icon=id::FiEdit width="18" height="18" />
            </button>
            <A
                href="/notifications"
                attr:class=move || {
                    if act_notif == "notif" { "wf-dock-item active" } else { "wf-dock-item" }
                }
                attr:aria-label="通知"
            >
                <span class="wf-ico-rel">
                    <Icon icon=id::FiBell width="20" height="20" />
                    <Show when=move || { notifications.unread_notifications.get() > 0u32 }>
                        <span class="wf-badge wf-badge-dot" />
                    </Show>
                </span>
            </A>
            <div class="wf-ico-wrap">
                <button
                    class=move || {
                        if more_open.get() || act_more == "profile" || act_more == "settings" {
                            "wf-dock-item active"
                        } else {
                            "wf-dock-item"
                        }
                    }
                    on:click=move |_| more_open.update(|v| *v = !*v)
                    aria-label="その他"
                >
                    <Icon icon=id::FiMoreHorizontal width="20" height="20" />
                </button>
                <Show when=move || more_open.get()>
                    <div class="wf-menu-scrim" on:click=move |_| more_open.set(false) />
                    <div class="wf-pop wf-dock-pop" role="menu">
                        {move || {
                            let href = profile_path(me.get().as_ref());
                            view! {
                                <A href=href attr:class="wf-pop-item" on:click=move |_| more_open.set(false)>
                                    <Icon icon=id::FiUser width="16" height="16" />
                                    "プロフィール"
                                </A>
                            }
                        }}
                        <A href="/settings" attr:class="wf-pop-item" on:click=move |_| more_open.set(false)>
                            <Icon icon=id::FiSettings width="16" height="16" />
                            "設定"
                        </A>
                        <A href="/drive" attr:class="wf-pop-item" on:click=move |_| more_open.set(false)>
                            <Icon icon=id::FiFolder width="16" height="16" />
                            "ドライブ"
                        </A>
                    </div>
                </Show>
            </div>
        </nav>
    }
}

#[component]
pub fn TopBar(
    #[prop(into, optional)] title: Option<String>,
    #[prop(optional)] tabs: Option<Vec<(icondata::Icon, &'static str, &'static str, bool)>>,
) -> impl IntoView {
    let tabs = tabs.unwrap_or_default();
    let has_tabs = !tabs.is_empty();
    let show_title = title.as_ref().is_some_and(|t| !t.is_empty()) && !has_tabs;
    let title_text = title.unwrap_or_default();
    let navigate = use_navigate();

    let seg = tabs
        .into_iter()
        .map(|(icon, label, href, active)| {
            let nav = navigate.clone();
            view! {
                <span
                    class=if active { "wf-seg-item active" } else { "wf-seg-item" }
                    on:click=move |_| nav(href, Default::default())
                    role="button"
                    title=label
                    aria-label=label
                >
                    <Icon icon=icon width="16" height="16" />
                </span>
            }
        })
        .collect_view();

    if !show_title && !has_tabs {
        return view! { <></> }.into_any();
    }

    view! {
        <header class="wf-topbar">
            {if show_title {
                view! { <h1 class="wf-title">{title_text}</h1> }.into_any()
            } else {
                view! { <></> }.into_any()
            }}
            {if has_tabs {
                view! { <div class="wf-seg">{seg}</div> }.into_any()
            } else {
                view! { <></> }.into_any()
            }}
        </header>
    }
    .into_any()
}
