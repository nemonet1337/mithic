use leptos::prelude::*;
use leptos_router::components::A;

use super::avatar::{Avatar, AvatarSize};
use super::mfm::MfmText;
use crate::models::{Note, NoteVisibility};

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
    let extra_class = if flat { " border-0 bg-transparent" } else { "" };
    view! {
        <article class=format!("post-card{}", extra_class)>
            <A href=route.clone() attr:class="flex-shrink-0">
                <Avatar user=author_avatar size=AvatarSize::Md />
            </A>
            <div class="post-main">
                <div class="post-header">
                    <A href=route attr:class="flex items-center gap-2 hover:underline">
                        <span class="post-author-name">{author_name}</span>
                        <span class="post-author-handle">{author_handle}</span>
                        <span class="post-time">"· " {created_at}</span>
                    </A>
                    <A href=note_href attr:class="btn btn-ghost btn-xs btn-circle opacity-40 hover:opacity-100">
                        "···"
                    </A>
                </div>
                <PostBody content=note.content.clone() cw=note.cw.clone() />
                <Show when=move || has_attachments>
                    <div class="rounded-xl overflow-hidden bg-base-200 h-48 flex items-center justify-center text-xs font-mono opacity-40 mt-2">
                        "メディア"
                    </div>
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
                    <div class="cw-box">
                        <span class="badge badge-warning badge-sm font-mono">"CW"</span>
                        <strong class="flex-1 text-sm">{cw_text}</strong>
                        <button
                            class="btn btn-ghost btn-xs rounded-full"
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
                <MfmText text=content.clone() />
            </Show>
        </div>
    }
}

#[component]
pub fn PostActions(note: Note) -> impl IntoView {
    view! {
        <div class="post-actions">
            <button class="btn btn-ghost btn-xs gap-1 hover:text-info">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                </svg>
                "返信"
            </button>
            <button class="btn btn-ghost btn-xs gap-1 hover:text-success">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>
                </svg>
                {note.renote_count.to_string()}
            </button>
            <button class="btn btn-ghost btn-xs gap-1 hover:text-error">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
                </svg>
                "REACT"
            </button>
            {note.reactions.into_iter().map(|r| view! {
                <button class=move || {
                    if r.reacted_by_me {
                        "btn btn-xs badge badge-primary gap-1"
                    } else {
                        "btn btn-xs badge badge-outline gap-1"
                    }
                }>
                    {r.emoji} " " {r.count.to_string()}
                </button>
            }).collect_view()}
            <span class="ml-auto font-mono text-xs opacity-30">{format!("↪ {}", note.quote_count)}</span>
        </div>
    }
}
