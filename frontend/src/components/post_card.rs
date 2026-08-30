use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

use super::avatar::{Avatar, AvatarSize};
use super::confirm_dialog::ConfirmDialog;
use super::markdown::MarkdownText;
use super::note_menu::{NoteMenu, NoteMenuAction};
use super::reaction_picker::ReactionPicker;
use super::toast::{ToastKind, ToastStore};
use crate::models::{Note, NoteVisibility};
use crate::store::{AuthStore, ComposeStore, StreamStore};
use crate::time::{date_label, relative_label};
use shared::MediaAttachment;

#[component]
pub fn PostCard(note: Note, #[prop(default = false)] flat: bool) -> impl IntoView {
    let renoter_name = note.author.name();
    let is_pure_renote =
        note.renote_id.is_some() && note.content.trim().is_empty() && note.renote.is_some();
    let is_quote =
        note.renote_id.is_some() && !note.content.trim().is_empty() && note.renote.is_some();
    let nested = note.renote.clone();

    // pure renote: バナー + ネストした元ノート
    if is_pure_renote {
        if let Some(inner) = nested {
            return view! {
                <article class="wf-entry">
                    <div class="wf-entry-body" style="grid-column:1/-1;">
                        <div class="wf-renote-banner">
                            <Icon icon=id::FiRepeat width="14" height="14" />
                            <span>{format!("{renoter_name} がリノート")}</span>
                        </div>
                        <PostCard note=*inner flat=true />
                    </div>
                </article>
            }
            .into_any();
        }
    }

    let auth = expect_context::<AuthStore>();
    let toast = expect_context::<ToastStore>();
    let stream = expect_context::<StreamStore>();
    let compose = expect_context::<ComposeStore>();

    let author = note.author.clone();
    let author_avatar = author.clone();
    let author_name = author.name();
    let author_handle = author.handle();
    let route = format!("/profile/{}", author.route_handle());
    let note_href = format!("/notes/{}", note.id);
    let note_id_for_menu = note.id.clone();
    let target_user_id = note.author.id.clone();
    let has_attachments = !note.attachments.is_empty();
    let created_at = note.created_at.clone();
    let delete_open = RwSignal::new(false);
    let preview_text = {
        let t = note.content.clone();
        let mut it = t.chars();
        let short: String = it.by_ref().take(80).collect();
        if it.next().is_some() {
            format!("{short}…")
        } else {
            short
        }
    };
    let preview_meta = format!("{} · {}", author_handle, date_label(&created_at));
    let visibility_mark = match note.visibility {
        NoteVisibility::Public => "",
        NoteVisibility::Home => "🏠",
        NoteVisibility::Followers => "🔒",
        NoteVisibility::Specified => "✉️",
    };
    let extra_class = if flat { " is-quote" } else { "" };
    let note_for_actions = note.clone();
    let quote_inner = if is_quote { nested } else { None };

    let menu_open = RwSignal::new(false);
    let is_own = {
        let author_id = note.author.id.clone();
        move || auth.me.get().map(|me| me.id == author_id).unwrap_or(false)
    };

    let on_menu_action = {
        let toast = toast;
        let note_id = note_id_for_menu.clone();
        let target_user_id = target_user_id.clone();
        Callback::new(move |action: NoteMenuAction| {
            let toast = toast;
            let note_id = note_id.clone();
            let target_user_id = target_user_id.clone();
            match action {
                NoteMenuAction::CopyLink => {
                    let origin = web_sys::window()
                        .and_then(|w| w.location().origin().ok())
                        .unwrap_or_default();
                    let url = format!("{origin}/notes/{note_id}");
                    toast.push(format!("リンク: {url}"), ToastKind::Info);
                }
                NoteMenuAction::Delete => delete_open.set(true),
                NoteMenuAction::Pin => {
                    let Some(tok) = auth.token.get_untracked() else {
                        toast.push("ログインが必要です", ToastKind::Error);
                        return;
                    };
                    wasm_bindgen_futures::spawn_local(async move {
                        match crate::api::notes::pin_note(&tok, &note_id).await {
                            Ok(()) => toast.push("ピン留めしました", ToastKind::Success),
                            Err(e) => toast.push(e.message, ToastKind::Error),
                        }
                    });
                }
                NoteMenuAction::Mute => {
                    let Some(tok) = auth.token.get_untracked() else {
                        toast.push("ログインが必要です", ToastKind::Error);
                        return;
                    };
                    wasm_bindgen_futures::spawn_local(async move {
                        match crate::api::users::mute(&tok, &target_user_id).await {
                            Ok(()) => toast.push("ミュートしました", ToastKind::Success),
                            Err(e) => toast.push(e.message, ToastKind::Error),
                        }
                    });
                }
                NoteMenuAction::Block => {
                    let Some(tok) = auth.token.get_untracked() else {
                        toast.push("ログインが必要です", ToastKind::Error);
                        return;
                    };
                    wasm_bindgen_futures::spawn_local(async move {
                        match crate::api::users::block(&tok, &target_user_id).await {
                            Ok(()) => toast.push("ブロックしました", ToastKind::Success),
                            Err(e) => toast.push(e.message, ToastKind::Error),
                        }
                    });
                }
                NoteMenuAction::Report => toast.push("通報は準備中です", ToastKind::Info),
            }
        })
    };

    let on_delete_confirm = {
        let toast = toast;
        let note_id = note_id_for_menu.clone();
        Callback::new(move |_| {
            let Some(tok) = auth.token.get_untracked() else {
                toast.push("ログインが必要です", ToastKind::Error);
                return;
            };
            let toast = toast;
            let note_id = note_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::notes::delete_note(&tok, &note_id).await {
                    Ok(()) => {
                        stream.mark_deleted(note_id);
                        toast.push("投稿を削除しました", ToastKind::Success);
                    }
                    Err(e) => toast.push(e.message, ToastKind::Error),
                }
            });
        })
    };

    view! {
        <article class=format!("wf-entry{}", extra_class)>
            <A href=route.clone() attr:class="wf-entry-avatar">
                <Avatar user=author_avatar size=AvatarSize::Md />
            </A>
            <div class="wf-entry-body">
                <div class="wf-entry-head">
                    <A href=route attr:class="flex items-center gap-2 hover:underline">
                        <span class="wf-entry-name">{author_name}</span>
                        <span class="wf-entry-handle">{format!("「{author_handle}」")}</span>
                        <span class="wf-entry-dot">"·"</span>
                        <span class="wf-entry-meta">
                            {visibility_mark} " " {date_label(&created_at)} " · " {relative_label(&created_at)}
                        </span>
                    </A>
                    <span class="wf-entry-menu relative">
                        <button
                            class="wf-btn wf-btn-ghost wf-btn-sm wf-btn-circle opacity-40 hover:opacity-100"
                            on:click=move |_| menu_open.update(|v| *v = !*v)
                        >
                            "···"
                        </button>
                        <NoteMenu
                            is_open=menu_open
                            is_own_note=Signal::derive(is_own)
                            on_action=on_menu_action
                            on_close=Callback::new(move |_| menu_open.set(false))
                        />
                    </span>
                </div>
                <PostBody content=note.content.clone() cw=note.cw.clone() />
                <Show when=move || has_attachments>
                    <MediaThumbs attachments=note.attachments.clone() />
                </Show>
                {quote_inner.map(|inner| {
                    view! {
                        <div class="wf-quote-wrap">
                            <PostCard note=*inner flat=true />
                        </div>
                    }
                })}
                <Show when=move || note.renote_id.is_some() && note.renote.is_none() && note.content.trim().is_empty()>
                    <div class="wf-entry-meta p-2">"元ノートを表示できません"</div>
                </Show>
                <PostActions note=note_for_actions compose=compose />
                <A href=note_href attr:class="sr-only">"詳細"</A>
            </div>
            <ConfirmDialog
                is_open=delete_open
                title="投稿を削除しますか？"
                body="この操作は取り消せません。投稿に紐づくリアクション・返信・引用も削除されます。"
                preview_meta=preview_meta
                preview=preview_text
                confirm_label="削除する"
                danger=true
                on_confirm=on_delete_confirm
                on_close=Callback::new(move |_| delete_open.set(false))
            />
        </article>
    }
    .into_any()
}

#[component]
fn MediaThumbs(attachments: Vec<MediaAttachment>) -> impl IntoView {
    let n = attachments.len().min(4);
    let grid = match n {
        1 => "wf-grid-1",
        2 => "wf-grid-2",
        3 => "wf-grid-3",
        _ => "wf-grid-4",
    };
    view! {
        <div class=format!("wf-media {}", grid)>
            {attachments.into_iter().take(4).map(|att| {
                let url = att.preview_url.clone().unwrap_or_else(|| att.url.clone());
                let alt = att.alt.clone().unwrap_or_default();
                view! {
                    <div class="wf-thumb aspect-video">
                        <img src=url alt=alt loading="lazy" />
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

#[component]
pub fn PostBody(content: String, cw: Option<String>) -> impl IntoView {
    let has_cw = cw.is_some();
    let cw_text = cw.unwrap_or_default();
    let expanded = RwSignal::new(!has_cw);
    view! {
        <div class="wf-entry-text">
            {if has_cw {
                view! {
                    <div class="wf-cw">
                        <span class="font-mono text-xs">"CW"</span>
                        <strong>{cw_text}</strong>
                        <button
                            class="wf-btn wf-btn-ghost wf-btn-sm ml-auto"
                            on:click=move |_| expanded.update(|v| *v = !*v)
                        >
                            {move || if expanded.get() { "隠す ▲" } else { "開く ▼" }}
                        </button>
                    </div>
                }.into_any()
            } else {
                ().into_any()
            }}
            <Show when=move || expanded.get()>
                <MarkdownText text=content.clone() />
            </Show>
        </div>
    }
}

#[component]
pub fn PostActions(note: Note, compose: ComposeStore) -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let toast = expect_context::<ToastStore>();
    let note_id = note.id.clone();
    let reply_id = note.id.clone();
    let reply_count = note.reply_count;
    let quote_count = note.quote_count;
    let author_handle = note.author.handle();
    let preview_text = {
        let t = note.content.clone();
        let mut it = t.chars();
        let short: String = it.by_ref().take(80).collect();
        if it.next().is_some() {
            format!("{short}…")
        } else {
            short
        }
    };
    let preview_meta = format!("{} · {}", author_handle, date_label(&note.created_at));
    let renote_count = RwSignal::new(note.renote_count);
    let reactions = RwSignal::new(note.reactions.clone());
    let react_open = RwSignal::new(false);
    let renote_open = RwSignal::new(false);
    let busy = RwSignal::new(false);

    let apply_reaction = {
        let toast = toast;
        let note_id = note_id.clone();
        Callback::new(move |emoji: String| {
            let Some(tok) = auth.token.get_untracked() else {
                toast.push("ログインが必要です", ToastKind::Error);
                return;
            };
            let toast = toast;
            let note_id = note_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::notes::add_reaction(&tok, &note_id, &emoji).await {
                    Ok(list) => reactions.set(list),
                    Err(e) => toast.push(e.message, ToastKind::Error),
                }
            });
        })
    };

    let confirm_renote = {
        let toast = toast;
        let note_id = note_id.clone();
        Callback::new(move |_| {
            if busy.get_untracked() {
                return;
            }
            let Some(tok) = auth.token.get_untracked() else {
                toast.push("ログインが必要です", ToastKind::Error);
                return;
            };
            busy.set(true);
            let toast = toast;
            let note_id = note_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::notes::renote(&tok, &note_id).await {
                    Ok(_) => {
                        renote_count.update(|c| *c = c.saturating_add(1));
                        toast.push("リノートしました", ToastKind::Success);
                    }
                    Err(e) => toast.push(e.message, ToastKind::Error),
                }
                busy.set(false);
            });
        })
    };

    view! {
        <div class="wf-actions relative">
            <button
                class="wf-react-btn"
                on:click=move |_| compose.open_reply(reply_id.clone())
            >
                <Icon icon=id::FiMessageSquare width="15" height="15" />
                {reply_count.to_string()}
            </button>
            <button
                class="wf-react-btn"
                disabled=move || busy.get()
                on:click=move |_| renote_open.set(true)
            >
                <Icon icon=id::FiRepeat width="15" height="15" />
                {move || renote_count.get().to_string()}
            </button>
            <button
                class="wf-react-btn"
                on:click=move |_| react_open.update(|v| *v = !*v)
            >
                <Icon icon=id::FiSmile width="15" height="15" />
                "＋REACT"
            </button>
            <ReactionPicker
                is_open=react_open
                on_select=apply_reaction
                on_close=Callback::new(move |_| react_open.set(false))
            />
            {move || reactions.get().into_iter().map(|r| {
                let on = r.reacted_by_me;
                let emoji = r.emoji.clone();
                view! {
                    <button
                        class=if on { "wf-pill on" } else { "wf-pill" }
                        on:click=move |_| apply_reaction.run(emoji.clone())
                    >
                        {format!("{} {}", r.emoji, r.count)}
                    </button>
                }
            }).collect_view()}
            <span class="wf-entry-meta ml-auto">
                {move || format!("↻ {} · ↪ {}", renote_count.get(), quote_count)}
            </span>
        </div>
        <ConfirmDialog
            is_open=renote_open
            title="リノートしますか？"
            body="元の投稿があなたのフォロワーのタイムラインに表示されます。元の公開範囲がホーム/フォロワー限定でも、あなた経由で広がる可能性があります。取り消す場合はリノートを削除してください。"
            preview_meta=preview_meta
            preview=preview_text
            confirm_label="リノートする"
            danger=true
            on_confirm=confirm_renote
            on_close=Callback::new(move |_| renote_open.set(false))
        />
    }
}
