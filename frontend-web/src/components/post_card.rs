use leptos::prelude::*;
use leptos_router::components::A;

use super::avatar::{Avatar, AvatarSize};
use super::mfm::MfmText;
use crate::models::{Note, NoteVisibility};

#[component]
pub fn PostCard(note: Note, #[prop(default = false)] flat: bool) -> impl IntoView {
    let author = note.author.clone();
    let route = format!("/{}", author.route_handle());
    let note_href = format!("/notes/{}", note.id);
    let has_attachments = !note.attachments.is_empty();
    let card_class = if flat { "wf-entry flat" } else { "wf-entry" };
    let date = if note.created_at.contains('m') {
        "NOW"
    } else {
        "MAY·10"
    };
    view! {
        <article class=card_class>
            <div class="wf-stamp accent">
                <span class="wf-stamp-date">{date}</span>
                <span class="wf-stamp-time">{note.created_at.clone()}</span>
            </div>
            <div class="wf-grow post-main">
                <div class="wf-spread post-header">
                    <A href=route attr:class="wf-row post-author">
                        <Avatar user=author.clone() size=AvatarSize::Sm />
                        <span class="wf-hand post-name">{author.name()}</span>
                        <span class="wf-mono post-handle">{author.handle()}</span>
                    </A>
                    <A href=note_href attr:class="wf-mono post-menu">"···"</A>
                </div>
                <PostBody content=note.content.clone() cw=note.cw.clone() />
                <Show when=move || has_attachments>
                    <div class="wf-media post-media">"media"</div>
                </Show>
                <PostActions note=note />
            </div>
        </article>
    }
}

#[component]
pub fn PostBody(content: String, cw: Option<String>) -> impl IntoView {
    let has_cw = cw.is_some();
    let cw_text = cw.unwrap_or_default();
    let expanded = RwSignal::new(!has_cw);
    view! {
        <div class="post-body">
            {if has_cw {
                view! {
                    <div class="wf-card dashed cw-box">
                        <span class="wf-label">"CW"</span>
                        <strong>{cw_text}</strong>
                        <button class="wf-btn sm ghost" on:click=move |_| expanded.update(|value| *value = !*value)>
                            {move || if expanded.get() { "隠す" } else { "開く" }}
                        </button>
                    </div>
                }.into_any()
            } else {
                view! {}.into_any()
            }}
            <Show when=move || expanded.get()>
                <MfmText text=content.clone() />
            </Show>
        </div>
    }
}

#[component]
pub fn PostActions(note: Note) -> impl IntoView {
    view! {
        <div class="wf-row post-actions">
            <button class="wf-btn sm">"↩ 返信"</button>
            <button class="wf-btn sm">{format!("↻ {}", note.renote_count)}</button>
            <button class="wf-btn sm">"＋ REACT"</button>
            {note.reactions.into_iter().map(|reaction| view! {
                <span class=move || if reaction.reacted_by_me { "wf-pill accent" } else { "wf-pill" }>
                    {reaction.emoji} " " {reaction.count.to_string()}
                </span>
            }).collect_view()}
            <span class="wf-mono action-summary">{format!("↪ {}", note.quote_count)}</span>
        </div>
    }
}
