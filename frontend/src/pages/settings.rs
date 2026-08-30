use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use crate::api::users::UpdateProfileRequest;
use crate::components::{Avatar, AvatarSize, Shell, ToastKind, ToastStore};
use crate::models::User;
use crate::store::AuthStore;
use shared::ProfileField;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let params = use_params_map();
    let section = move || {
        let raw = params.read().get("section").unwrap_or_else(|| "プロフィール".into());
        match raw.as_str() {
            "パスワード" | "プライバシー" | "通知" | "テーマ" | "QR" => raw,
            _ => "プロフィール".into(),
        }
    };

    let nav = [
        ("プロフィール", "アカウント"),
        ("パスワード", "アカウント"),
        ("QR", "アカウント"),
        ("プライバシー", "プライバシー"),
        ("通知", "通知"),
        ("テーマ", "表示"),
    ];

    view! {
        <Shell active="settings">
            <div class="wf-settings-layout flex" style="height:100%;overflow:hidden;">
                <aside class="wf-rail" style="width:200px;flex-shrink:0;">
                    <span class="wf-title" style="font-size:18px;">"設定"</span>
                    {nav.into_iter().map(|(item, group)| {
                        view! {
                            <div>
                                <span class="wf-rail-tag" style="display:block;margin:8px 0 4px;">{group}</span>
                                <A href=format!("/settings/{item}")
                                    attr:class=move || if section() == item { "wf-pop-item active" } else { "wf-pop-item" }>
                                    {item}
                                </A>
                            </div>
                        }
                    }).collect_view()}
                </aside>
                <main class="wf-scroll p-6" style="flex:1;">
                    {move || match section().as_str() {
                        "テーマ" => view! { <ThemeSection /> }.into_any(),
                        "通知" => view! { <PushSection /> }.into_any(),
                        "パスワード" => view! { <PasswordSection /> }.into_any(),
                        "プライバシー" => view! { <PrivacySection /> }.into_any(),
                        "QR" => view! { <QrSection /> }.into_any(),
                        _ => view! { <ProfileSection /> }.into_any(),
                    }}
                </main>
            </div>
        </Shell>
    }
}

#[component]
fn ProfileSection() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let toast = expect_context::<ToastStore>();
    let me = auth.me;
    let token = auth.token;

    let display_name = RwSignal::new(String::new());
    let bio = RwSignal::new(String::new());
    let location = RwSignal::new(String::new());
    let birthday = RwSignal::new(String::new());
    let lang = RwSignal::new("ja".to_string());
    let followed_message = RwSignal::new(String::new());
    let reaction_acceptance = RwSignal::new(String::new());
    let is_bot = RwSignal::new(false);
    let is_cat = RwSignal::new(false);
    let avatar_url = RwSignal::new(String::new());
    let banner_url = RwSignal::new(String::new());
    let fields = RwSignal::new(vec![
        ProfileField::default(),
        ProfileField::default(),
        ProfileField::default(),
        ProfileField::default(),
    ]);
    let save_busy = RwSignal::new(false);

    let hydrate = move |user: User| {
        display_name.set(user.display_name.clone().unwrap_or_default());
        bio.set(user.bio.clone().unwrap_or_default());
        location.set(user.location.clone().unwrap_or_default());
        birthday.set(user.birthday.clone().unwrap_or_default());
        lang.set(user.lang.clone().unwrap_or_else(|| "ja".into()));
        followed_message.set(user.followed_message.clone().unwrap_or_default());
        reaction_acceptance.set(user.reaction_acceptance.clone().unwrap_or_default());
        is_bot.set(user.is_bot);
        is_cat.set(user.is_cat);
        avatar_url.set(user.avatar_url.clone().unwrap_or_default());
        banner_url.set(user.banner_url.clone().unwrap_or_default());
        let mut rows = user.fields;
        while rows.len() < 4 {
            rows.push(ProfileField::default());
        }
        fields.set(rows);
    };

    Effect::new(move |_| {
        if let Some(user) = me.get() {
            hydrate(user);
        }
    });

    let upload_image = move |for_banner: bool, ev: web_sys::Event| {
        let Some(t) = ev.target() else { return };
        let Ok(inp) = t.dyn_into::<HtmlInputElement>() else {
            return;
        };
        let Some(files) = inp.files() else { return };
        let Some(file) = files.item(0) else { return };
        let Some(tok) = token.get_untracked() else { return };
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::drive::upload(&tok, &file).await {
                Ok(stored) => {
                    if for_banner {
                        banner_url.set(stored.url);
                    } else {
                        avatar_url.set(stored.url);
                    }
                    toast.push("画像をアップロードしました", ToastKind::Success);
                }
                Err(e) => toast.push(e.user_message(), ToastKind::Error),
            }
        });
    };

    let on_save = move |_| {
        if save_busy.get_untracked() {
            return;
        }
        let Some(tok) = token.get_untracked() else {
            return;
        };
        let req = UpdateProfileRequest {
            display_name: Some(display_name.get_untracked()),
            bio: Some(bio.get_untracked()),
            location: Some(location.get_untracked()),
            birthday: Some(birthday.get_untracked()),
            lang: Some(lang.get_untracked()),
            followed_message: Some(followed_message.get_untracked()),
            reaction_acceptance: Some(reaction_acceptance.get_untracked()),
            is_bot: Some(is_bot.get_untracked()),
            is_cat: Some(is_cat.get_untracked()),
            avatar_url: Some(avatar_url.get_untracked()),
            banner_url: Some(banner_url.get_untracked()),
            fields: Some(fields.get_untracked()),
            is_locked: None,
        };
        save_busy.set(true);
        let toast = toast;
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::users::update_me(&tok, &req).await {
                Ok(updated) => {
                    auth.me.set(Some(updated));
                    toast.push("プロフィールを保存しました", ToastKind::Success);
                }
                Err(e) => toast.push(e.user_message(), ToastKind::Error),
            }
            save_busy.set(false);
        });
    };

    view! {
        <span class="wf-entry-meta">"アカウント / プロフィール"</span>
        <h1 class="wf-title mt-1 mb-6">"プロフィール設定"</h1>
        <div class="flex flex-col gap-5 max-w-xl">
            <div
                class="wf-profile-banner rounded-wf"
                style=move || {
                    let url = banner_url.get();
                    if url.is_empty() { String::new() } else { format!("background-image:url('{url}');background-size:cover;") }
                }
            >
                <div class="p-3 flex justify-end">
                    <label class="wf-btn wf-btn-sm">
                        "バナーを変更"
                        <input class="sr-only" type="file" accept="image/*" on:change=move |e| upload_image(true, e) />
                    </label>
                </div>
            </div>
            <div class="flex items-center gap-3">
                {move || me.get().map(|u| view! { <Avatar user=u size=AvatarSize::Xl /> })}
                <label class="wf-btn wf-btn-sm">
                    "アバターを変更"
                    <input class="sr-only" type="file" accept="image/*" on:change=move |e| upload_image(false, e) />
                </label>
            </div>
            <Field label="表示名">
                <input class="wf-input" prop:value=move || display_name.get() on:input=move |e| display_name.set(event_target_value(&e)) />
            </Field>
            <Field label="自己紹介">
                <textarea class="wf-input" style="height:96px;resize:vertical;" prop:value=move || bio.get() on:input=move |e| bio.set(event_target_value(&e)) />
            </Field>
            <Field label="場所">
                <input class="wf-input" prop:value=move || location.get() on:input=move |e| location.set(event_target_value(&e)) />
            </Field>
            <Field label="誕生日">
                <input class="wf-input" type="date" prop:value=move || birthday.get() on:input=move |e| birthday.set(event_target_value(&e)) />
            </Field>
            <Field label="言語">
                <select class="wf-select w-full" on:change=move |e| lang.set(event_target_value(&e))>
                    <option value="ja" selected=move || lang.get() == "ja">"日本語"</option>
                    <option value="en" selected=move || lang.get() == "en">"English"</option>
                </select>
            </Field>
            <div>
                <div class="wf-spread mb-2">
                    <span class="wf-entry-meta">"追加情報（最大16）"</span>
                    <button class="wf-btn wf-btn-ghost wf-btn-sm" on:click=move |_| {
                        fields.update(|rows| {
                            if rows.len() < 16 {
                                rows.push(ProfileField::default());
                            }
                        });
                    }>"＋ 行を追加"</button>
                </div>
                {move || fields.get().into_iter().enumerate().map(|(i, field)| {
                    view! {
                        <div class="flex gap-2 mb-2">
                            <input class="wf-input" placeholder="ラベル" prop:value=field.name.clone()
                                on:input=move |e| {
                                    let v = event_target_value(&e);
                                    fields.update(|rows| { if let Some(r) = rows.get_mut(i) { r.name = v; } });
                                } />
                            <input class="wf-input" placeholder="内容" prop:value=field.value.clone()
                                on:input=move |e| {
                                    let v = event_target_value(&e);
                                    fields.update(|rows| { if let Some(r) = rows.get_mut(i) { r.value = v; } });
                                } />
                        </div>
                    }
                }).collect_view()}
            </div>
            <Field label="フォローされたときのメッセージ">
                <input class="wf-input" maxlength="200" prop:value=move || followed_message.get() on:input=move |e| followed_message.set(event_target_value(&e)) />
            </Field>
            <Field label="リアクションの受け入れ">
                <select class="wf-select w-full" on:change=move |e| reaction_acceptance.set(event_target_value(&e))>
                    <option value="" selected=move || reaction_acceptance.get().is_empty()>"すべて"</option>
                    <option value="likeOnly" selected=move || reaction_acceptance.get() == "likeOnly">"いいねのみ"</option>
                    <option value="likeOnlyForRemote" selected=move || reaction_acceptance.get() == "likeOnlyForRemote">"リモートはいいねのみ"</option>
                    <option value="nonSensitiveOnly" selected=move || reaction_acceptance.get() == "nonSensitiveOnly">"NSFW 以外"</option>
                    <option value="nonSensitiveOnlyForLocalLikeOnlyForRemote" selected=move || reaction_acceptance.get() == "nonSensitiveOnlyForLocalLikeOnlyForRemote">"ローカルは NSFW 以外 / リモートはいいねのみ"</option>
                </select>
            </Field>
            <div class="wf-card flex flex-col gap-3">
                <span class="wf-entry-meta">"高度な設定"</span>
                <label class="wf-spread text-sm">
                    <span>"猫として設定する"</span>
                    <input class="wf-check" type="checkbox" prop:checked=move || is_cat.get() on:change=move |_| is_cat.update(|v| *v = !*v) />
                </label>
                <label class="wf-spread text-sm">
                    <span>"Bot として設定する"</span>
                    <input class="wf-check" type="checkbox" prop:checked=move || is_bot.get() on:change=move |_| is_bot.update(|v| *v = !*v) />
                </label>
            </div>
            <div class="flex justify-end gap-2">
                <button class="wf-btn wf-btn-primary" disabled=move || save_busy.get() on:click=on_save>
                    {move || if save_busy.get() { "保存中…" } else { "保存" }}
                </button>
            </div>
        </div>
    }
}

#[component]
fn Field(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <label class="flex flex-col gap-1 w-full">
            <span class="wf-entry-meta">{label}</span>
            {children()}
        </label>
    }
}

#[component]
fn PasswordSection() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let toast = expect_context::<ToastStore>();
    let current = RwSignal::new(String::new());
    let next = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    view! {
        <span class="wf-entry-meta">"アカウント / パスワード"</span>
        <h1 class="wf-title mt-1 mb-6">"パスワード変更"</h1>
        <div class="flex flex-col gap-3 max-w-md">
            <input class="wf-input" type="password" placeholder="現在のパスワード" prop:value=move || current.get() on:input=move |e| current.set(event_target_value(&e)) />
            <input class="wf-input" type="password" placeholder="新しいパスワード" prop:value=move || next.get() on:input=move |e| next.set(event_target_value(&e)) />
            <button class="wf-btn wf-btn-primary" disabled=move || busy.get() on:click=move |_| {
                let Some(tok) = auth.token.get_untracked() else { return };
                busy.set(true);
                let cur = current.get_untracked();
                let newp = next.get_untracked();
                wasm_bindgen_futures::spawn_local(async move {
                    match crate::api::users::change_password(&tok, &cur, &newp).await {
                        Ok(()) => toast.push("パスワードを変更しました", ToastKind::Success),
                        Err(e) => toast.push(e.user_message(), ToastKind::Error),
                    }
                    busy.set(false);
                });
            }>"変更する"</button>
        </div>
    }
}

#[component]
fn PrivacySection() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let toast = expect_context::<ToastStore>();
    let locked = RwSignal::new(false);
    Effect::new(move |_| {
        if let Some(u) = auth.me.get() {
            locked.set(u.is_locked);
        }
    });
    let blocks = RwSignal::new(Vec::<User>::new());
    let mutes = RwSignal::new(Vec::<User>::new());
    Effect::new(move |_| {
        let Some(tok) = auth.token.get() else { return };
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(list) = crate::api::users::list_blocks(&tok).await {
                blocks.set(list);
            }
            if let Ok(list) = crate::api::users::list_mutes(&tok).await {
                mutes.set(list);
            }
        });
    });
    view! {
        <span class="wf-entry-meta">"プライバシー"</span>
        <h1 class="wf-title mt-1 mb-6">"公開範囲とブロック"</h1>
        <div class="flex flex-col gap-4 max-w-lg">
            <div class="wf-card wf-spread">
                <div>
                    <div class="text-sm font-semibold">"フォローを承認制にする"</div>
                    <div class="wf-entry-meta">"ロックアカウント"</div>
                </div>
                <input class="wf-check" type="checkbox" prop:checked=move || locked.get() on:change=move |_| {
                    locked.update(|v| *v = !*v);
                    let Some(tok) = auth.token.get_untracked() else { return };
                    let value = locked.get_untracked();
                    wasm_bindgen_futures::spawn_local(async move {
                        let req = UpdateProfileRequest { is_locked: Some(value), ..Default::default() };
                        match crate::api::users::update_me(&tok, &req).await {
                            Ok(u) => { auth.me.set(Some(u)); toast.push("保存しました", ToastKind::Success); }
                            Err(e) => toast.push(e.user_message(), ToastKind::Error),
                        }
                    });
                } />
            </div>
            <div class="wf-card">
                <span class="wf-entry-meta">"ブロック中"</span>
                {move || {
                    let list = blocks.get();
                    if list.is_empty() {
                        view! { <p class="wf-entry-meta mt-2">"まだいません"</p> }.into_any()
                    } else {
                        list.into_iter().map(|u| view! { <div class="text-sm mt-1">{u.handle()}</div> }).collect_view().into_any()
                    }
                }}
            </div>
            <div class="wf-card">
                <span class="wf-entry-meta">"ミュート中"</span>
                {move || {
                    let list = mutes.get();
                    if list.is_empty() {
                        view! { <p class="wf-entry-meta mt-2">"まだいません"</p> }.into_any()
                    } else {
                        list.into_iter().map(|u| view! { <div class="text-sm mt-1">{u.handle()}</div> }).collect_view().into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn PushSection() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let toast = expect_context::<ToastStore>();
    let token = auth.token;
    let push_busy = RwSignal::new(false);
    view! {
        <span class="wf-entry-meta">"アカウント / 通知"</span>
        <h1 class="wf-title mt-1 mb-6">"通知設定"</h1>
        <div class="wf-card max-w-md flex flex-col gap-3">
            <span class="text-sm font-semibold">"Web プッシュ通知"</span>
            <p class="wf-entry-meta text-sm">"ブラウザがバックグラウンドでも、メンションやフォローなどを通知します。サーバー側で VAPID 鍵が必要です。"</p>
            <div class="flex gap-2">
                <button class="wf-btn wf-btn-primary" disabled=move || push_busy.get() on:click=move |_| {
                    let Some(tok) = token.get_untracked() else { toast.push("ログインが必要です", ToastKind::Error); return; };
                    push_busy.set(true);
                    wasm_bindgen_futures::spawn_local(async move {
                        #[cfg(target_arch = "wasm32")]
                        {
                            match crate::api::push::enable_browser_push(&tok).await {
                                Ok(()) => toast.push("ブラウザ通知を有効にしました", ToastKind::Success),
                                Err(e) => toast.push(e, ToastKind::Error),
                            }
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        { let _ = tok; toast.push("WASM のみ対応", ToastKind::Info); }
                        push_busy.set(false);
                    });
                }>{move || if push_busy.get() { "処理中…" } else { "有効にする" }}</button>
                <button class="wf-btn wf-btn-ghost" disabled=move || push_busy.get() on:click=move |_| {
                    let Some(tok) = token.get_untracked() else { return };
                    push_busy.set(true);
                    wasm_bindgen_futures::spawn_local(async move {
                        match crate::api::push::unsubscribe(&tok).await {
                            Ok(()) => toast.push("プッシュ通知の登録を解除しました", ToastKind::Success),
                            Err(e) => toast.push(e.user_message(), ToastKind::Error),
                        }
                        push_busy.set(false);
                    });
                }>"解除"</button>
            </div>
        </div>
    }
}

#[component]
fn ThemeSection() -> impl IntoView {
    let theme = RwSignal::new(
        LocalStorage::get::<String>("mithic.theme").unwrap_or_else(|_| "night".into()),
    );
    let set_theme = move |t: &'static str| {
        let is_dark = match t {
            "light" => false,
            "auto" => web_sys::window()
                .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
                .map(|mql| mql.matches())
                .unwrap_or(true),
            _ => true,
        };
        theme.set(t.into());
        let _ = LocalStorage::set("mithic.theme", t);
        if let Some(html) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            let class = html.class_name();
            let next = if is_dark {
                if class.split_whitespace().any(|c| c == "dark") {
                    class
                } else if class.is_empty() {
                    "dark".into()
                } else {
                    format!("{class} dark")
                }
            } else {
                class
                    .split_whitespace()
                    .filter(|c| *c != "dark")
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            let _ = html.set_class_name(&next);
            let _ = html.set_attribute("data-theme", if is_dark { "night" } else { "light" });
        }
    };
    view! {
        <span class="wf-entry-meta">"表示 / テーマ"</span>
        <h1 class="wf-title mt-1 mb-6">"テーマ設定"</h1>
        <div class="wf-card max-w-md flex flex-row items-center justify-between">
            <span class="text-sm font-semibold">"テーマ"</span>
            <div class="flex gap-1">
                <button class=move || if theme.get() == "light" { "wf-btn wf-btn-primary wf-btn-sm" } else { "wf-btn wf-btn-ghost wf-btn-sm" } on:click=move |_| set_theme("light")>"ライト"</button>
                <button class=move || if theme.get() == "dark" || theme.get() == "night" { "wf-btn wf-btn-primary wf-btn-sm" } else { "wf-btn wf-btn-ghost wf-btn-sm" } on:click=move |_| set_theme("night")>"ダーク"</button>
                <button class=move || if theme.get() == "auto" { "wf-btn wf-btn-primary wf-btn-sm" } else { "wf-btn wf-btn-ghost wf-btn-sm" } on:click=move |_| set_theme("auto")>"自動"</button>
            </div>
        </div>
    }
}

#[component]
fn QrSection() -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let url = move || {
        let origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_default();
        let handle = auth.me.get().map(|u| u.username).unwrap_or_default();
        format!("{origin}/profile/{handle}")
    };
    view! {
        <span class="wf-entry-meta">"アカウント / QR"</span>
        <h1 class="wf-title mt-1 mb-6">"プロフィール QR"</h1>
        <div class="wf-card max-w-md flex flex-col gap-3">
            <p class="text-sm">"この URL を共有するとプロフィールを開けます。"</p>
            <code class="font-mono text-xs break-all">{url}</code>
            <button class="wf-btn wf-btn-sm" on:click=move |_| {
                let _ = url();
            }>"この URL をメモしてください"</button>
        </div>
    }
}
