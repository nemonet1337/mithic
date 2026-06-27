use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn MarkdownText(text: String) -> impl IntoView {
    let html = shared::markdown::render_markdown(&text);
    view! {
        <span class="markdown-text" inner_html=html />
    }
}