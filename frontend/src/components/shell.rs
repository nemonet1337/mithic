use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

use super::avatar::{Avatar, AvatarAccent, AvatarSize};
use crate::store::{AuthStore, ComposeStore, NotificationStore};

// ===========================================================
// Shell — full-bleed timeline canvas + floating chrome
// ===========================================================
#[component]
pub fn Shell(
    #[prop(into)] active: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="app-root wf-shell" id="app-root">
            // タイムラインが全面。クロームは浮遊オーバーレイ。
            <main class="wf-main">
                <div class="wf-main-inner">
                    {children()}
                </div>
            </main>
            <AppChrome active=active.clone() />
            <MobileDock active=active />
        </div>
    }
}

// ===========================================================
// Floating chrome (corners)
// ===========================================================
#[component]
fn AppChrome(#[prop(into)] active: String) -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    let notifications = expect_context::<NotificationStore>();
    let auth = expect_context::<AuthStore>();
    let me = auth.me;
    let navigate = leptos_router::hooks::use_navigate();
    let active_for_notif = active.clone();

    let open_menu = RwSignal::new(Option::<&'static str>::None);
    let close_menus = move || open_menu.set(None);
    let toggle = move |id: &'static str| {
        open_menu.update(|cur| {
            *cur = if *cur == Some(id) { None } else { Some(id) };
        });
    };

    let profile_href = move || {
        if let Some(user) = me.get() {
            format!("/profile/{}", user.username)
        } else {
            "/you".into()
        }
    };

    let any_menu = Signal::derive(move || open_menu.get().is_some());

    view! {
        <div class="wf-chrome" aria-label="アプリ操作">
            // 左上: ホーム
            <A href="/" attr:class="wf-chrome-brand" attr:aria-label="ホーム" attr:title="ホーム">
                <span class="wf-stamp">"m"</span>
            </A>

            // 右上: 操作クラスタ
            <div class="wf-chrome-cluster">
                <div class="wf-ico-wrap">
                    <button
                        class=move || {
                            if open_menu.get() == Some("nav") {
                                "wf-ico-btn active"
                            } else {
                                "wf-ico-btn"
                            }
                        }
                        on:click=move |_| toggle("nav")
                        aria-label="メニュー"
                        aria-expanded=move || (open_menu.get() == Some("nav")).to_string()
                        title="メニュー"
                    >
                        <Icon icon=id::FiMenu width="18" height="18" />
                    </button>
                    <Show when=move || open_menu.get() == Some("nav")>
                        <div class="wf-pop wf-nav-pop wf-nav-pop-end" role="menu">
                            <A href="/" attr:class="wf-pop-item" on:click=move |_| close_menus()>
                                <Icon icon=id::FiHome width="16" height="16" />
                                "ホーム"
                            </A>
                            <A href="/search" attr:class="wf-pop-item" on:click=move |_| close_menus()>
                                <Icon icon=id::FiSearch width="16" height="16" />
                                "検索"
                            </A>
                            <A href="/dm" attr:class="wf-pop-item" on:click=move |_| close_menus()>
                                <Icon icon=id::FiMail width="16" height="16" />
                                "メッセージ"
                                <Show when=move || { notifications.unread_messages.get() > 0u32 }>
                                    <span class="wf-badge" style="margin-left:auto;">
                                        {move || notifications.unread_messages.get().to_string()}
                                    </span>
                                </Show>
                            </A>
                            {move || {
                                let href = profile_href();
                                view! {
                                    <A href=href attr:class="wf-pop-item" on:click=move |_| close_menus()>
                                        <Icon icon=id::FiUser width="16" height="16" />
                                        "プロフィール"
                                    </A>
                                }
                            }}
                            <A href="/drive" attr:class="wf-pop-item" on:click=move |_| close_menus()>
                                <Icon icon=id::FiFolder width="16" height="16" />
                                "ドライブ"
                            </A>
                        </div>
                    </Show>
                </div>

                <A
                    href="/notifications"
                    attr:class=move || {
                        if active_for_notif == "notif" {
                            "wf-ico-btn active"
                        } else {
                            "wf-ico-btn"
                        }
                    }
                    attr:aria-label="通知"
                    attr:title="通知"
                >
                    <span class="wf-ico-rel">
                        <Icon icon=id::FiBell width="18" height="18" />
                        <Show when=move || { notifications.unread_notifications.get() > 0u32 }>
                            <span class="wf-badge wf-badge-dot">
                                {move || {
                                    let n = notifications.unread_notifications.get();
                                    if n > 99 {
                                        "99+".into()
                                    } else {
                                        n.to_string()
                                    }
                                }}
                            </span>
                        </Show>
                    </span>
                </A>

                <button
                    class="wf-ico-btn wf-ico-compose"
                    on:click=move |_| compose.open()
                    aria-label="投稿"
                    title="投稿"
                >
                    <Icon icon=id::FiEdit width="16" height="16" />
                </button>

                <div class="wf-ico-wrap">
                    <button
                        class="wf-ico-btn wf-ico-avatar"
                        on:click=move |_| toggle("account")
                        aria-label="アカウント"
                        aria-expanded=move || (open_menu.get() == Some("account")).to_string()
                        title="アカウント"
                    >
                        {move || {
                            me.get().map(|u| {
                                view! {
                                    <Avatar user=u size=AvatarSize::Sm accent=AvatarAccent::None />
                                }
                            })
                        }}
                    </button>
                    <Show when=move || open_menu.get() == Some("account")>
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
                                let href = profile_href();
                                view! {
                                    <A href=href attr:class="wf-pop-item" on:click=move |_| close_menus()>
                                        <Icon icon=id::FiUser width="16" height="16" />
                                        "プロフィール"
                                    </A>
                                }
                            }}
                            <A href="/settings" attr:class="wf-pop-item" on:click=move |_| close_menus()>
                                <Icon icon=id::FiSettings width="16" height="16" />
                                "設定"
                            </A>
                            <hr class="wf-spine-rule" style="margin:4px 0;" />
                            <button
                                class="wf-pop-item danger"
                                on:click={
                                    let auth = auth.clone();
                                    let navigate = navigate.clone();
                                    move |_| {
                                        open_menu.set(None);
                                        let token = auth.token.get_untracked();
                                        auth.logout();
                                        if let Some(tok) = token {
                                            wasm_bindgen_futures::spawn_local(async move {
                                                let _ = crate::api::auth::logout(&tok).await;
                                            });
                                        }
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

            <Show when=move || any_menu.get()>
                <div class="wf-menu-scrim" on:click=move |_| close_menus() />
            </Show>
        </div>
    }
}

// ===========================================================
// Mobile floating dock
// ===========================================================
#[component]
fn MobileDock(#[prop(into)] active: String) -> impl IntoView {
    let compose = expect_context::<ComposeStore>();
    let notifications = expect_context::<NotificationStore>();
    let auth = expect_context::<AuthStore>();
    let me = auth.me;
    let more_open = RwSignal::new(false);

    let profile_href = move || {
        if let Some(user) = me.get() {
            format!("/profile/{}", user.username)
        } else {
            "/you".into()
        }
    };

    let act_home = active.clone();
    let act_search = active.clone();
    let act_notif = active.clone();
    let act_more = active.clone();

    view! {
        <nav class="wf-dock" aria-label="メインナビ">
            <A
                href="/"
                attr:class=move || {
                    if act_home == "home" {
                        "wf-dock-item active"
                    } else {
                        "wf-dock-item"
                    }
                }
                attr:aria-label="ホーム"
                attr:title="ホーム"
            >
                <Icon icon=id::FiHome width="20" height="20" />
            </A>
            <A
                href="/search"
                attr:class=move || {
                    if act_search == "search" {
                        "wf-dock-item active"
                    } else {
                        "wf-dock-item"
                    }
                }
                attr:aria-label="検索"
                attr:title="検索"
            >
                <Icon icon=id::FiSearch width="20" height="20" />
            </A>
            <button
                class="wf-dock-compose"
                on:click=move |_| compose.open()
                aria-label="投稿"
                title="投稿"
            >
                <Icon icon=id::FiEdit width="18" height="18" />
            </button>
            <A
                href="/notifications"
                attr:class=move || {
                    if act_notif == "notif" {
                        "wf-dock-item active"
                    } else {
                        "wf-dock-item"
                    }
                }
                attr:aria-label="通知"
                attr:title="通知"
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
                        if more_open.get()
                            || act_more == "profile"
                            || act_more == "settings"
                            || act_more == "dm"
                        {
                            "wf-dock-item active"
                        } else {
                            "wf-dock-item"
                        }
                    }
                    on:click=move |_| more_open.update(|v| *v = !*v)
                    aria-label="その他"
                    title="その他"
                >
                    <Icon icon=id::FiMoreHorizontal width="20" height="20" />
                </button>
                <Show when=move || more_open.get()>
                    <div class="wf-menu-scrim" on:click=move |_| more_open.set(false) />
                    <div class="wf-pop wf-dock-pop" role="menu">
                        {move || {
                            let href = profile_href();
                            view! {
                                <A href=href attr:class="wf-pop-item" on:click=move |_| more_open.set(false)>
                                    <Icon icon=id::FiUser width="16" height="16" />
                                    "プロフィール"
                                </A>
                            }
                        }}
                        <A href="/dm" attr:class="wf-pop-item" on:click=move |_| more_open.set(false)>
                            <Icon icon=id::FiMail width="16" height="16" />
                            "メッセージ"
                            <Show when=move || { notifications.unread_messages.get() > 0u32 }>
                                <span class="wf-badge" style="margin-left:auto;">
                                    {move || notifications.unread_messages.get().to_string()}
                                </span>
                            </Show>
                        </A>
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

// ===========================================================
// TopBar — floating chips / soft page label
// ===========================================================
#[component]
pub fn TopBar(
    #[prop(into, optional)] title: Option<String>,
    #[prop(optional)] tabs: Option<Vec<(icondata::Icon, &'static str, &'static str, bool)>>,
) -> impl IntoView {
    let tabs = tabs.unwrap_or_default();
    let has_tabs = !tabs.is_empty();
    let show_title = title.as_ref().is_some_and(|t| !t.is_empty()) && !has_tabs;
    let title_text = title.unwrap_or_default();
    let navigate = leptos_router::hooks::use_navigate();

    let seg = tabs
        .into_iter()
        .map(|(icon, label, href, active)| {
            let nav = navigate.clone();
            view! {
                <span
                    class=if active {
                        "wf-seg-item active"
                    } else {
                        "wf-seg-item"
                    }
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
        <header class=if has_tabs { "wf-topbar wf-topbar-float" } else { "wf-topbar wf-topbar-soft" }>
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
