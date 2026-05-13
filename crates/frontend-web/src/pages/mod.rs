use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::components::{Avatar, AvatarSize, MfmText, PostCard, Shell, TopBar};
use crate::models::{NotificationType, sample_notes, sample_notifications, sample_user};
use crate::store::{AuthStore, NotificationStore, stream::connect_stream};

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
    let notes = RwSignal::new(sample_notes());
    let auth = expect_context::<AuthStore>();
    let notifications = expect_context::<NotificationStore>();
    let active = match kind {
        TimelineKind::Home => "/",
        TimelineKind::Local => "/local",
        TimelineKind::Global => "/global",
    };
    let title = match kind {
        TimelineKind::Home => "ホーム",
        TimelineKind::Local => "ローカル",
        TimelineKind::Global => "グローバル",
    };

    Effect::new(move |_| {
        if let Some(token) = auth.token.get() {
            connect_stream(token, notes, notifications.unread_notifications);
        }
    });

    view! {
        <Shell active="home">
            <TopBar title=title folio="01" tabs=TIMELINE_TABS.to_vec() active_tab=active />
            <section class="timeline-list">
                {move || notes.get().into_iter().map(|note| view! { <PostCard note=note /> }).collect_view()}
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
            <TopBar title="通知" folio="03" />
            <div class="wf-spread notification-actions">
                <div class="wf-tabs"><span class="t on">"すべて"</span><span class="t">"@メンション"</span><span class="t">"いいね"</span><span class="t">"フォロー"</span></div>
                <button class="wf-btn sm" on:click=move |_| notification_store.mark_notifications_read()>"すべて既読"</button>
            </div>
            <section class="timeline-list">
                {notifications.into_iter().map(|notification| {
                    let sender = notification.sender.clone();
                    let note = notification.note.clone();
                    let unread_class = if notification.is_read { "notification-card" } else { "notification-card unread" };
                    let kind = match notification.notification_type {
                        NotificationType::Reaction => format!("{} がリアクションしました", notification.reaction.unwrap_or_default()),
                        NotificationType::Reply => "返信が届きました".to_string(),
                        NotificationType::Follow => "フォローされました".to_string(),
                        _ => "新しい通知".to_string(),
                    };
                    view! {
                        <article class=unread_class>
                            <div class="unread-dot" />
                            {sender.map(|user| view! { <Avatar user=user size=AvatarSize::Sm /> }).into_view()}
                            <div class="wf-grow">
                                <div class="wf-spread"><strong>{kind}</strong><span class="wf-mono muted-text">{notification.created_at}</span></div>
                                {note.map(|note| view! { <blockquote class="notif-preview"><MfmText text=note.content /></blockquote> }).into_view()}
                                <div class="wf-row notif-actions"><button class="wf-btn sm ghost">"返信"</button><button class="wf-btn sm">"開く"</button></div>
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
    let conversation_id = params.read().get("id");
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
                            <A href=format!("/messages/{id}") attr:class=move || if active { "dm-row active" } else { "dm-row" }>
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
    let handle = move || params.read().get("handle").unwrap_or_else(|| "hana".into());
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
    let auth = expect_context::<AuthStore>();
    view! {
        <Shell active="home" right_rail=false>
            <section class="login-panel wf-card raised">
                <span class="wf-label">"[ MITHIC LOGIN ]"</span>
                <h1 class="wf-hand login-title">"おかえりなさい"</h1>
                <label class="field"><span class="wf-label">"ハンドル"</span><input placeholder="@hana" /></label>
                <label class="field"><span class="wf-label">"パスワード"</span><input type="password" placeholder="••••••••" /></label>
                <button class="wf-btn accent full" on:click=move |_| auth.login("dev-token".into(), sample_user("you", "You"))>"ログイン"</button>
                <A href="/" attr:class="wf-btn ghost full">"タイムラインへ"</A>
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
