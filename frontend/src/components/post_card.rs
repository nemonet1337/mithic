use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

use super::avatar::{Avatar, AvatarSize};
use super::markdown::MarkdownText;
use super::note_menu::{NoteMenu, NoteMenuAction};
use super::reaction_picker::ReactionPicker;
use super::toast::{ToastKind, ToastStore};
use crate::models::{Note, NoteVisibility};
use crate::store::AuthStore;
use shared::MediaAttachment;

#[component]
pub fn PostCard(note: Note, #[prop(default = false)] flat: bool) -> impl IntoView {
    let renoter_name = note.author.name();
    let is_pure_renote = note.renote_id.is_some()
        && note.content.trim().is_empty()
        && note.renote.is_some();
    let is_quote = note.renote_id.is_some()
        && !note.content.trim().is_empty()
        && note.renote.is_some();
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

    let author = note.author.clone();
    let author_avatar = author.clone();
    let author_name = author.name();
    let author_handle = author.handle();
    let route = format!("/profile/{}", author.route_handle());
    let note_href = format!("/notes/{}", note.id);
    let note_id_for_menu = note.id.clone();
    let has_attachments = !note.attachments.is_empty();
    let created_at = note.created_at.clone();
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
        move || {
            auth.me
                .get()
                .map(|me| me.id == author_id)
                .unwrap_or(false)
        }
    };

    let on_menu_action = {
        let toast = toast;
        let note_id = note_id_for_menu.clone();
        Callback::new(move |action: NoteMenuAction| {
            let toast = toast;
            let note_id = note_id.clone();
            match action {
                NoteMenuAction::CopyLink => {
                    let origin = web_sys::window()
                        .and_then(|w| w.location().origin().ok())
                        .unwrap_or_default();
                    let url = format!("{origin}/notes/{note_id}");
                    toast.push(format!("リンク: {url}"), ToastKind::Info);
                }
                NoteMenuAction::Delete => {
                    let Some(tok) = auth.token.get_untracked() else {
                        toast.push("ログインが必要です", ToastKind::Error);
                        return;
                    };
                    wasm_bindgen_futures::spawn_local(async move {
                        match crate::api::notes::delete_note(&tok, &note_id).await {
                            Ok(()) => toast.push("投稿を削除しました", ToastKind::Success),
                            Err(e) => toast.push(e.message, ToastKind::Error),
                        }
                    });
                }
                NoteMenuAction::Pin => toast.push("ピン留めは準備中です", ToastKind::Info),
                NoteMenuAction::Mute => toast.push("ミュートは準備中です", ToastKind::Info),
                NoteMenuAction::Block => toast.push("ブロックは準備中です", ToastKind::Info),
                NoteMenuAction::Report => toast.push("通報は準備中です", ToastKind::Info),
                NoteMenuAction::Unpin => {}
            }
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
                        <span class="wf-entry-handle">{format!("\"@{}\"", author_handle)}</span>
                        <span class="wf-entry-dot">"·"</span>
                        <span class="wf-entry-meta">{visibility_mark} " " {created_at}</span>
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
                <PostActions note=note_for_actions />
                <A href=note_href attr:class="sr-only">"詳細"</A>
            </div>
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
pub fn PostActions(note: Note) -> impl IntoView {
    let auth = expect_context::<AuthStore>();
    let toast = expect_context::<ToastStore>();
    let note_id = note.id.clone();
    let reply_count = note.reply_count;
    let renote_count = RwSignal::new(note.renote_count);
    let reactions = RwSignal::new(note.reactions.clone());
    let react_open = RwSignal::new(false);
    let busy = RwSignal::new(false);

    let do_renote = {
        let toast = toast;
        let note_id = note_id.clone();
        move |_| {
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
        }
    };

    let on_react = {
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
                    Ok(()) => {
                        reactions.update(|list| {
                            if let Some(r) = list.iter_mut().find(|r| r.emoji == emoji) {
                                r.count = r.count.saturating_add(1);
                                r.reacted_by_me = true;
                            } else {
                                list.push(shared::ReactionSummary {
                                    emoji: emoji.clone(),
                                    count: 1,
                                    reacted_by_me: true,
                                });
                            }
                        });
                        toast.push("リアクションしました", ToastKind::Success);
                    }
                    Err(e) => toast.push(e.message, ToastKind::Error),
                }
            });
        })
    };

    view! {
        <div class="wf-actions relative">
            <button class="wf-react-btn">
                <Icon icon=id::FiMessageSquare width="15" height="15" />
                {reply_count.to_string()}
            </button>
            <button class="wf-react-btn" disabled=move || busy.get() on:click=do_renote>
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
                on_select=on_react
                on_close=Callback::new(move |_| react_open.set(false))
            />
            {move || reactions.get().into_iter().map(|r| {
                let on = r.reacted_by_me;
                view! {
                    <button class=if on { "wf-pill on" } else { "wf-pill" }>
                        {format!("{} {}", r.emoji, r.count)}
                    </button>
                }
            }).collect_view()}
            <span class="wf-entry-meta ml-auto">{format!("↪ {}", note.quote_count)}</span>
        </div>
    }
}
