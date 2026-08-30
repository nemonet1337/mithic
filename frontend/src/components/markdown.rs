use leptos::prelude::*;

#[component]
pub fn MarkdownText(text: String) -> impl IntoView {
    let html = shared::markdown::render_markdown(&text);
    view! {
        <span class="markdown-text" inner_html=html />
    }
}
