use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};

use crate::components::{Avatar, AvatarSize, MfmText, PostCard, Shell, TopBar};
use crate::models::{sample_notes, sample_notifications, sample_user, Note, NotificationType};
use crate::store::{stream::connect_stream, AuthStore, NotificationStore};

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
    let auth        = expect_context::<AuthStore>();
    let notifications = expect_context::<NotificationStore>();
    let notes       = RwSignal::<Vec<Note>>::new(vec![]);
    let is_loading  = RwSignal::new(false);
    let has_more    = RwSignal::new(true);

    let kind_str = match kind {
        TimelineKind::Home   => "home",
        TimelineKind::Local  => "local",
        TimelineKind::Global => "global",
    };
    let active = match kind {
        TimelineKind::Home   => "/",
        TimelineKind::Local  => "/local",
        TimelineKind::Global => "/global",
    };
    let title = match kind {
        TimelineKind::Home   => "ホーム",
        TimelineKind::Local  => "ローカル",
        TimelineKind::Global => "グローバル",
    };

    // 初回ロード
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

    // WebSocket でリアルタイム先頭挿入
    Effect::new(move |_| {
        if let Some(token) = auth.token.get() {
            connect_stream(token, notes, notifications.unread_notifications);
        }
    });

    let load_more = move || {
        let token  = auth.token.get_untracked();
        let oldest = notes.with_untracked(|v| v.last().map(|n| n.id.clone()));
        if let (Some(tok), Some(id)) = (token, oldest) {
            is_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::notes::fetch_timeline(&tok, kind_str, Some(&id)).await {
                    Ok(mut more) => {
                        if more.is_empty() { has_more.set(false); }
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
            <section class="timeline-list">
                <For
                    each=move || notes.get()
                    key=|note| note.id.clone()
                    children=|note| view! { <PostCard note=note /> }
                />
                <Show when=move || is_loading.get()>
                    <div class="timeline-loading">"読み込み中…"</div>
                </Show>
                <Show when=move || !is_loading.get() && has_more.get() && !notes.get().is_empty()>
                    <button class="wf-btn ghost full load-more-btn"
                        on:click=move |_| load_more()>
                        "さらに読み込む"
                    </button>
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
            <div class="detail-grid">
                <section class="detail-main">
                    <PostCard note=current.clone() flat=true />
                    <div class="wf-card reply-composer">
                        <span class="wf-label">"返信先 " {current.author.handle()}</span>
                        <textarea class="reply-input" placeholder=format!("{} への返信", current.author.handle()) />
                        <div class="wf-spread">
                            <div class="wf-row"><button class="wf-btn sm ghost">"添付"</button><button class="wf-btn sm ghost">"絵文字"</button></div>
                            <button class="wf-btn accent">"返信"</button>
                        </div>
                    </div>
                    {notes.into_iter().skip(1).map(|note| view! { <PostCard note=note /> }).collect_view()}
                </section>
                <aside class="detail-side wf-col">
                    <section class="wf-card side-card">
                        <span class="wf-label">"[ REACTIONS ]"</span>
                        <div class="reaction-grid">
                            {current.reactions.iter().map(|reaction| view! {
                                <span class="wf-pill accent2">{reaction.emoji.clone()} " " {reaction.count.to_string()}</span>
                            }).collect_view()}
                        </div>
                    </section>
                    <section class="wf-card side-card">
                        <span class="wf-label">"[ QUOTES ]"</span>
                        <p>"引用は元投稿を埋め込み表示します。"</p>
                    </section>
                </aside>
            </div>
        </Shell>
    }
}

#[component]
pub fn NotificationsPage() -> impl IntoView {
    let notification_store = expect_context::<NotificationStore>();
    let notifications = sample_notifications();
    view! {
        <Shell active="notif">
            <div class="wf-spread notification-actions">
                <span class="wf-hand" style="font-size:24px">"アクティビティ"</span>
                <div class="wf-row">
                    <Show when=move || notification_store.unread_notifications.get() > 0>
                        <span class="wf-pill accent">
                            "未読 " {move || notification_store.unread_notifications.get().to_string()}
                        </span>
                    </Show>
                    <button class="wf-btn sm ghost"
                        on:click=move |_| notification_store.mark_notifications_read()>
                        "既読に"
                    </button>
                </div>
            </div>
            <div class="wf-tabs" style="padding:0 16px">
                <span class="t on">"すべて"</span>
                <span class="t">"@メンション"</span>
                <span class="t">"いいね"</span>
                <span class="t">"フォロー"</span>
            </div>
            <section class="timeline-list">
                {notifications.into_iter().map(|notification| {
                    let sender = notification.sender.clone();
                    let note   = notification.note.clone();
                    let unread_class = if notification.is_read { "notification-card" } else { "notification-card unread" };
                    let (kind_label, action_view) = match notification.notification_type {
                        NotificationType::Reaction => (
                            format!("{} があなたの投稿に", notification.reaction.as_deref().unwrap_or("リアクション")),
                            view! {}.into_any(),
                        ),
                        NotificationType::Reply => (
                            "が返信しました".into(),
                            view! {
                                <div class="wf-row notif-actions">
                                    <button class="wf-btn sm ghost">"返信"</button>
                                    <button class="wf-btn sm">"開く"</button>
                                </div>
                            }.into_any(),
                        ),
                        NotificationType::Follow => (
                            "があなたをフォローしました".into(),
                            view! {
                                <button class="wf-btn sm primary">"フォローバック"</button>
                            }.into_any(),
                        ),
                        NotificationType::Renote => (
                            "がリノートしました".into(),
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
                                <div class="wf-row notif-actions">
                                    <button class="wf-btn sm">"承認"</button>
                                    <button class="wf-btn sm ghost">"拒否"</button>
                                </div>
                            }.into_any(),
                        ),
                        NotificationType::FollowRequestAccepted => (
                            "があなたのフォローリクエストを承認しました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::PollEnded => (
                            "のアンケートが終了しました".into(),
                            view! {}.into_any(),
                        ),
                        NotificationType::UserSignup => (
                            "が登録しました".into(),
                            view! {}.into_any(),
                        ),
                    };
                    view! {
                        <article class=unread_class>
                            <div class="unread-dot" />
                            {sender.map(|user| view! { <Avatar user=user size=AvatarSize::Sm /> }).into_view()}
                            <div class="wf-grow">
                                <div class="wf-spread">
                                    <strong>{kind_label}</strong>
                                    <span class="wf-mono muted-text">{notification.created_at}</span>
                                </div>
                                {note.map(|n| view! {
                                    <blockquote class="notif-preview"><MfmText text=n.content /></blockquote>
                                }).into_view()}
                                {action_view}
                            </div>
                        </article>
                    }
                }).collect_view()}
            </section>
        </Shell>
    }
}

#[component]
pub fn SearchPage() -> impl IntoView {
    let query = use_query_map();
    let term = move || query.read().get("q").unwrap_or_default();
    view! {
        <Shell active="search">
            <TopBar title="検索 / 発見" folio="04" />
            <section class="search-hero wf-card raised">
                <span class="wf-label">"COMMAND · PALETTE"</span>
                <div class="command-input"><span>"⌘K"</span><input placeholder="投稿・ユーザー・タグを検索" prop:value=move || term() /></div>
                <div class="wf-row search-pills">
                    {vec!["#art", "#tech", "#books", "#music", "#food", "#photo"].into_iter().map(|tag| view! { <A href=format!("/search?tag={}", tag.trim_start_matches('#')) attr:class="wf-pill">{tag}</A> }).collect_view()}
                </div>
            </section>
            <section class="discover-grid">
                {sample_notes().into_iter().take(3).map(|note| view! { <PostCard note=note /> }).collect_view()}
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
                <aside class="dm-list-pane">
                    <div class="wf-spread dm-list-head"><span class="wf-hand dm-title">"DM"</span><button class="wf-btn icon sm">"+"</button></div>
                    <div class="wf-input dashed">"検索"</div>
                    {vec![
                        ("hana", "Hana K.", "@hana", "余白について話そう", "2m", true),
                        ("riku", "Riku M.", "@riku", "OK 送りました", "14m", false),
                        ("aya", "Aya T.", "@aya", "📚", "1h", false),
                        ("design", "Group · design", "3 人", "Ken: たしかに", "3h", false),
                    ].into_iter().map(|(id, name, handle, last, time, unread)| {
                        let active = selected == id;
                        view! {
                            <A href=format!("/dm/{id}") attr:class=move || if active { "dm-row active" } else { "dm-row" }>
                                <div class="wf-av sm accent" />
                                <div class="wf-col wf-grow"><div class="wf-spread"><span class="wf-hand dm-name">{name}</span><span class="wf-mono muted-text">{time}</span></div><span class="dm-last">{handle} "· " {last}</span></div>
                                <Show when=move || unread><span class="unread-dot static-dot" /></Show>
                            </A>
                        }
                    }).collect_view()}
                </aside>
                <main class="dm-conversation">
                    <div class="wf-spread dm-conv-head"><div class="wf-row"><div class="wf-av sm accent" /><div class="wf-col"><span class="wf-hand dm-name">"Hana K."</span><span class="wf-mono muted-text">"@hana · オンライン"</span></div></div><button class="wf-btn icon ghost">"···"</button></div>
                    <div class="dm-messages">
                        <span class="wf-mono date-sep">"— 今日 —"</span>
                        <MessageBubble mine=false text="ワイヤーの粒度ってどう決めてる？" />
                        <MessageBubble mine=true text="決めすぎないように。会話が生まれる粒度。" />
                        <MessageBubble mine=false text="なるほど。じゃあ余白について話そう" />
                    </div>
                    <div class="dm-input"><div class="wf-input lg"><span class="wf-grow muted-text">"メッセージを入力…"</span><span>"📎 😊"</span></div></div>
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
    let handle = move || {
        params
            .read()
            .get("username")
            .unwrap_or_else(|| "hana".into())
    };
    let user = sample_user("hana", "Hana K.");
    view! {
        <Shell active="profile">
            <section class="profile-head wf-card raised">
                <div class="profile-banner" />
                <div class="profile-main">
                    <Avatar user=user.clone() size=AvatarSize::Xl />
                    <div class="wf-grow">
                        <span class="wf-label">"ACTIVITYPUB · " {move || if handle().contains('@') { "REMOTE" } else { "LOCAL" }}</span>
                        <h1 class="wf-hand profile-name">{user.name()}</h1>
                        <span class="wf-mono muted-text">{move || format!("@{}", handle())}</span>
                        <p>{user.bio.clone().unwrap_or_default()}</p>
                        <div class="wf-row stats"><span>{user.notes_count.to_string()} " 投稿"</span><span>{user.followers_count.to_string()} " フォロワー"</span><span>{user.following_count.to_string()} " フォロー"</span></div>
                    </div>
                    <div class="wf-row"><button class="wf-btn accent">"フォロー"</button><button class="wf-btn icon ghost">"···"</button></div>
                </div>
            </section>
            <div class="wf-tabs profile-tabs"><span class="t on">"投稿"</span><span class="t">"返信"</span><span class="t">"メディア"</span><span class="t">"いいね"</span></div>
            <section class="timeline-list">{sample_notes().into_iter().map(|note| view! { <PostCard note=note /> }).collect_view()}</section>
        </Shell>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let groups = vec![
        (
            "アカウント",
            vec!["プロフィール", "メール", "パスワード", "連携アカウント"],
        ),
        ("プライバシー", vec!["公開範囲", "ブロック", "ミュート"]),
        ("通知", vec!["プッシュ", "メール", "メンション"]),
        ("表示", vec!["テーマ", "言語", "タイムゾーン"]),
        ("データ", vec!["エクスポート", "削除"]),
        ("2段階認証", vec!["TOTP", "SMS"]),
    ];
    view! {
        <Shell active="settings" right_rail=false>
            <div class="settings-layout">
                <aside class="settings-nav">
                    <span class="wf-hand settings-title">"設定"</span>
                    {groups.into_iter().map(|(group, items)| view! {
                        <section><span class="wf-label">{group}</span><div class="wf-stack settings-group">{items.into_iter().map(|item| view! { <A href=format!("/settings/{}", item) attr:class="settings-link">{item}<span>"›"</span></A> }).collect_view()}</div></section>
                    }).collect_view()}
                </aside>
                <main class="settings-content">
                    <span class="wf-label">"アカウント / プロフィール"</span>
                    <h1 class="wf-hand settings-main-title">"プロフィール設定"</h1>
                    <div class="settings-form">
                        <div class="wf-row"><div class="wf-av xl accent" /><div class="wf-col"><button class="wf-btn sm">"画像を変更"</button><button class="wf-btn sm ghost">"削除"</button></div></div>
                        <Field label="表示名" value="Hana K." />
                        <Field label="ハンドル" value="@hana" />
                        <label class="field"><span class="wf-label">"自己紹介"</span><textarea>"UI設計と植物。決めない自由を残す。"</textarea></label>
                        <div class="wf-row form-actions"><button class="wf-btn ghost">"キャンセル"</button><button class="wf-btn primary">"保存"</button></div>
                    </div>
                </main>
            </div>
        </Shell>
    }
}

#[component]
fn Field(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! { <label class="field"><span class="wf-label">{label}</span><input value=value /></label> }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth     = expect_context::<AuthStore>();
    let handle   = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let remember = RwSignal::new(false);
    let show_pw  = RwSignal::new(false);
    let error    = RwSignal::<Option<String>>::new(None);
    let loading  = RwSignal::new(false);
    let navigate = use_navigate();

    let on_submit = move |_| {
        let h = handle.get();
        let p = password.get();

        if h.trim().is_empty() {
            error.set(Some("ハンドルまたはメールアドレスを入力してください".into()));
            return;
        }
        if p.len() < 8 {
            error.set(Some("パスワードは8文字以上です".into()));
            return;
        }
        error.set(None);
        loading.set(true);

        let auth2 = auth.clone();
        let nav2  = navigate.clone();
        wasm_bindgen_futures::spawn_local(async move {
            use crate::api::auth::{LoginRequest, login};
            let req = LoginRequest { handle: h, password: p, remember: remember.get() };
            match login(&req).await {
                Ok(pair) => {
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
                <div class="auth-aside-hatch" />
                <span class="auth-aside-tag">"[ LOG IN · 01 ]"</span>
                <h1 class="wf-hand auth-aside-title">
                    "ようこそ、"<br />
                    <span class="wf-uline">"mithic"</span>" へ。"
                </h1>
                <p class="auth-aside-sub">"決めない自由を残したまま、再開しましょう。"</p>
                <div class="auth-postmark">
                    <div class="auth-postmark-inner" />
                    <span class="wf-mono" style="font-size:9px;letter-spacing:.14em">"EST."</span>
                    <span class="wf-hand" style="font-size:22px;line-height:1">"2024"</span>
                    <span class="wf-mono" style="font-size:8px;letter-spacing:.1em;margin-top:2px">"TOKYO"</span>
                </div>
                <div class="wf-mono" style="font-size:9px;color:var(--ink-3);margin-top:18px">
                    "— mithic · signal not noise —"
                </div>
            </aside>

            <div class="auth-form-area">
                <div class="auth-form-inner wf-stack" style="gap:12px">
                    <div class="auth-mark">
                        <span class="wf-mark-bracket" style="font-size:28px">"["</span>
                        <span class="wf-mark-glyph"  style="font-size:28px">"m"</span>
                        <span class="wf-mark-bracket" style="font-size:28px">"]"</span>
                        <span class="wf-hand" style="font-size:30px;margin-left:6px">"mithic"</span>
                    </div>

                    <span class="wf-label">"[ 既存アカウント / SIGN IN ]"</span>
                    <h2 class="wf-hand" style="font-size:28px;margin:4px 0 0">"ログイン"</h2>

                    <Show when=move || error.get().is_some()>
                        <div class="auth-error">
                            <span class="wf-pill accent" style="font-size:9px">"[ ERROR ]"</span>
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>

                    <div>
                        <div class="wf-spread" style="margin-bottom:4px">
                            <span class="wf-label">"サーバ"</span>
                            <span class="wf-mono" style="font-size:9px;color:var(--ink-3)">"変更 ▾"</span>
                        </div>
                        <div class="wf-input lg">
                            <span class="wf-mono" style="color:var(--ink-3);margin-right:6px">"@"</span>
                            <span style="flex:1;color:var(--ink)">"mithic.social"</span>
                            <span class="wf-pill accent2" style="font-size:9px">"● 接続中"</span>
                        </div>
                    </div>

                    <label class="field">
                        <span class="wf-label">"ハンドル / メール"</span>
                        <input class="wf-input lg"
                            placeholder="@hana"
                            prop:value=move || handle.get()
                            on:input=move |e| handle.set(event_target_value(&e))
                        />
                    </label>

                    <div>
                        <div class="wf-spread" style="margin-bottom:4px">
                            <span class="wf-label">"パスワード"</span>
                            <span class="wf-mono" style="font-size:9px;color:var(--accent)">"忘れた場合"</span>
                        </div>
                        <div class="wf-input lg">
                            <input
                                style="flex:1;border:0;background:transparent;outline:0;color:var(--ink)"
                                prop:type=move || if show_pw.get() { "text" } else { "password" }
                                placeholder="••••••••"
                                prop:value=move || password.get()
                                on:input=move |e| password.set(event_target_value(&e))
                            />
                            <button class="wf-btn icon ghost sm" on:click=move |_| show_pw.update(|v| *v = !*v)>
                                {move || if show_pw.get() { "🔒" } else { "👁" }}
                            </button>
                        </div>
                    </div>

                    <div class="wf-spread">
                        <label class="wf-row" style="gap:6px;font-size:11px;cursor:pointer">
                            <input type="checkbox"
                                prop:checked=move || remember.get()
                                on:change=move |_| remember.update(|v| *v = !*v)
                            />
                            "このデバイスを記憶"
                        </label>
                        <span class="wf-pill" style="font-size:10px">"🔐 2FA有効"</span>
                    </div>

                    <button class="wf-btn accent full lg"
                        disabled=move || loading.get()
                        on:click=on_submit>
                        {move || if loading.get() { "認証中…" } else { "ログイン →" }}
                    </button>

                    <p style="font-size:11px;color:var(--ink-3);text-align:center">
                        "はじめての方は "
                        <A href="/signup" attr:class="wf-tag">"新規登録 →"</A>
                    </p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn SignupPage() -> impl IntoView {
    let signup_handle    = RwSignal::new(String::new());
    let display_name     = RwSignal::new(String::new());
    let email            = RwSignal::new(String::new());
    let password         = RwSignal::new(String::new());
    let password_confirm = RwSignal::new(String::new());
    let agreed_age       = RwSignal::new(false);
    let agreed_tos       = RwSignal::new(false);
    let handle_available = RwSignal::<Option<bool>>::new(None);
    let error            = RwSignal::<Option<String>>::new(None);

    // ハンドル可用性チェック (簡易デバウンス)
    Effect::new(move |_| {
        let h = signup_handle.get();
        if h.len() < 3 { handle_available.set(None); return; }
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;
            if signup_handle.get_untracked() != h { return; }
            match crate::api::users::check_handle(&h).await {
                Ok(r)  => handle_available.set(Some(r.available)),
                Err(_) => handle_available.set(None),
            }
        });
    });

    let pw_strength = Memo::new(move |_| {
        let p = password.get();
        let mut score = 0u8;
        if p.len() >= 8  { score += 1; }
        if p.len() >= 12 { score += 1; }
        if p.chars().any(|c| c.is_ascii_uppercase())    { score += 1; }
        if p.chars().any(|c| c.is_ascii_punctuation())  { score += 1; }
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
                <div class="auth-aside-hatch" />
                <span class="auth-aside-tag">"[ SIGN UP · 01 ]"</span>
                <h1 class="wf-hand auth-aside-title">"アカウントを作成しましょう。"</h1>
                <p class="auth-aside-sub">"mithic はオープンな分散型 SNS です。ActivityPub でつながります。"</p>
                <div class="wf-mono" style="font-size:9px;color:var(--ink-3);margin-top:auto">
                    "— mithic · signal not noise —"
                </div>
            </aside>

            <div class="auth-form-area">
                <div class="auth-form-inner wf-stack" style="gap:12px">
                    <div class="signup-progress">
                        <div class="signup-progress-seg done" />
                        <div class="signup-progress-seg" />
                        <div class="signup-progress-seg" />
                    </div>

                    <span class="wf-label">"[ STEP 1/3 · 登録情報 ]"</span>
                    <h2 class="wf-hand" style="font-size:28px;margin:4px 0 0">"新規登録"</h2>

                    <Show when=move || error.get().is_some()>
                        <div class="auth-error">
                            <span class="wf-pill accent" style="font-size:9px">"[ ERROR ]"</span>
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>

                    <div>
                        <div class="wf-spread" style="margin-bottom:4px">
                            <span class="wf-label">"ハンドル"</span>
                            {move || match handle_available.get() {
                                Some(true)  => view! { <span class="wf-pill accent2" style="font-size:9px">"✓ 利用可"</span> }.into_any(),
                                Some(false) => view! { <span class="wf-pill accent"  style="font-size:9px">"✗ 使用中"</span> }.into_any(),
                                None        => view! { <span></span> }.into_any(),
                            }}
                        </div>
                        <input class="wf-input lg"
                            placeholder="@hana"
                            prop:value=move || signup_handle.get()
                            on:input=move |e| signup_handle.set(event_target_value(&e))
                        />
                    </div>

                    <label class="field">
                        <span class="wf-label">"表示名"</span>
                        <input class="wf-input lg"
                            placeholder="Hana K."
                            prop:value=move || display_name.get()
                            on:input=move |e| display_name.set(event_target_value(&e))
                        />
                    </label>

                    <label class="field">
                        <span class="wf-label">"メールアドレス"</span>
                        <input class="wf-input lg"
                            type="email"
                            placeholder="hana@example.com"
                            prop:value=move || email.get()
                            on:input=move |e| email.set(event_target_value(&e))
                        />
                    </label>

                    <div>
                        <span class="wf-label">"パスワード"</span>
                        <input class="wf-input lg"
                            type="password"
                            placeholder="••••••••"
                            prop:value=move || password.get()
                            on:input=move |e| password.set(event_target_value(&e))
                        />
                        <div class="pw-strength-bar">
                            {move || (1..=4u8).map(|i| {
                                let strength = pw_strength.get();
                                let cls = if strength >= i {
                                    format!("pw-strength-seg active-{i}")
                                } else {
                                    "pw-strength-seg".into()
                                };
                                view! { <div class=cls /> }
                            }).collect_view()}
                        </div>
                    </div>

                    <label class="field">
                        <span class="wf-label">"パスワード確認"</span>
                        <input class="wf-input lg"
                            type="password"
                            placeholder="••••••••"
                            prop:value=move || password_confirm.get()
                            on:input=move |e| password_confirm.set(event_target_value(&e))
                        />
                    </label>

                    <label class="wf-row" style="gap:6px;font-size:11px;cursor:pointer">
                        <input type="checkbox"
                            prop:checked=move || agreed_age.get()
                            on:change=move |_| agreed_age.update(|v| *v = !*v)
                        />
                        "私は13歳以上です"
                    </label>

                    <label class="wf-row" style="gap:6px;font-size:11px;cursor:pointer">
                        <input type="checkbox"
                            prop:checked=move || agreed_tos.get()
                            on:change=move |_| agreed_tos.update(|v| *v = !*v)
                        />
                        "利用規約に同意します"
                    </label>

                    <button class="wf-btn accent full lg"
                        disabled=move || !can_proceed.get()>
                        "次へ →"
                    </button>

                    <p style="font-size:11px;color:var(--ink-3);text-align:center">
                        "既にアカウントをお持ちの方は "
                        <A href="/login" attr:class="wf-tag">"ログイン →"</A>
                    </p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <Shell active="settings">
            <TopBar title="管理コンソール" folio="99" />
            <section class="wf-card raised">
                <h2 class="wf-hand">"Admin"</h2>
                <p>"P2で実装予定の管理画面です。"</p>
            </section>
        </Shell>
    }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <Shell active="home" right_rail=false>
            <section class="wf-card raised auth-gate">
                <span class="wf-label">"[ 404 ]"</span>
                <h1 class="wf-hand auth-gate-title">"ページが見つかりません"</h1>
                <A href="/" attr:class="wf-btn primary">"ホームへ戻る"</A>
            </section>
        </Shell>
    }
}
