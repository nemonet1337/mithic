use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use icondata as id;

use super::avatar::{Avatar, AvatarSize};
use super::markdown::MarkdownText;
use crate::models::{Note, NoteVisibility};
use shared::MediaAttachment;

#[component]
pub fn PostCard(note: Note, #[prop(default = false)] flat: bool) -> impl IntoView {
    let author = note.author.clone();
    let author_avatar = author.clone();
    let author_name = author.name();
    let author_handle = author.handle();
    let route = format!("/profile/{}", author.route_handle());
    let note_href = format!("/notes/{}", note.id);
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
                    <span class="wf-entry-menu">
                        <A href=note_href attr:class="wf-btn wf-btn-ghost wf-btn-sm wf-btn-circle opacity-40 hover:opacity-100">
                            "···"
                        </A>
                    </span>
                </div>
                <PostBody content=note.content.clone() cw=note.cw.clone() />
                <Show when=move || has_attachments>
                    <MediaThumbs attachments=note.attachments.clone() />
                </Show>
                <PostActions note=note_for_actions />
            </div>
        </article>
    }
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
                view! {}.into_any()
            }}
            <Show when=move || expanded.get()>
                <MarkdownText text=content.clone() />
            </Show>
        </div>
    }
}

#[component]
pub fn PostActions(note: Note) -> impl IntoView {
    let react_on = note
        .reactions
        .iter()
        .any(|r| r.reacted_by_me);
    view! {
        <div class="wf-actions">
            <button class="wf-react-btn">
                <Icon icon=id::FiMessageSquare width="15" height="15" />
                {note.reply_count.to_string()}
            </button>
            <button class="wf-react-btn">
                <Icon icon=id::FiRepeat width="15" height="15" />
                {note.renote_count.to_string()}
            </button>
            <button class=move || if react_on { "wf-react-btn on" } else { "wf-react-btn" }>
                <Icon icon=id::FiSmile width="15" height="15" />
                "＋REACT"
            </button>
            {note.reactions.into_iter().map(|r| {
                let on = r.reacted_by_me;
                view! {
                    <button class=move || if on { "wf-pill on" } else { "wf-pill" }>
                        {format!("{} {}", r.emoji, r.count)}
                    </button>
                }
            }).collect_view()}
            <span class="wf-entry-meta ml-auto">{format!("↪ {}", note.quote_count)}</span>
        </div>
    }
}
