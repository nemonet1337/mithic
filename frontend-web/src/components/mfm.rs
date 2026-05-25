use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn MfmText(text: String) -> impl IntoView {
    view! {
        <span class="mfm-text">
            {shared::mfm::parse(&text).into_iter().map(render_mfm_node).collect_view()}
        </span>
    }
}

pub fn render_mfm_node(node: shared::mfm::MfmNode) -> AnyView {
    match node {
        shared::mfm::MfmNode::Text(text) => view! { <span>{text}</span> }.into_any(),
        shared::mfm::MfmNode::Mention(acct) => view! {
            <A href=format!("/{}", acct) attr:class="text-accent">"@"{acct}</A>
        }
        .into_any(),
        shared::mfm::MfmNode::Hashtag(tag) => view! {
            <A href=format!("/search?tag={tag}") attr:class="text-accent">"#"{tag}</A>
        }
        .into_any(),
        shared::mfm::MfmNode::Url(url) => {
            let href = url.clone();
            view! {
                <a class="text-accent break-all" href=href target="_blank" rel="noreferrer">{url}</a>
            }
            .into_any()
        }
        shared::mfm::MfmNode::Bold(children) => view! {
            <strong>{children.into_iter().map(render_mfm_node).collect_view()}</strong>
        }
        .into_any(),
        shared::mfm::MfmNode::Italic(children) => view! {
            <em>{children.into_iter().map(render_mfm_node).collect_view()}</em>
        }
        .into_any(),
        shared::mfm::MfmNode::Emoji(name) => {
            view! { <span class="inline-emoji">{format!(":{name}:")}</span> }.into_any()
        }
        shared::mfm::MfmNode::InlineCode(code) => view! { <code>{code}</code> }.into_any(),
        shared::mfm::MfmNode::LineBreak => view! { <br /> }.into_any(),
    }
}
