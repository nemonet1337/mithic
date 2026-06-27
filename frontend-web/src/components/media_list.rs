use leptos::prelude::*;

use crate::components::MediaImage;
use crate::components::MediaVideo;
use crate::models::MediaAttachment;

#[component]
pub fn MediaList(attachments: Vec<MediaAttachment>) -> impl IntoView {
    if attachments.is_empty() {
        return ().into_any();
    }

    let count = attachments.len();
    let grid_class = match count {
        1 => "grid grid-cols-1 gap-2",
        2 => "grid grid-cols-2 gap-2",
        _ => "grid grid-cols-2 gap-2",
    };

    view! {
        <div class=grid_class>
            {attachments.into_iter().enumerate().map(|(i, att)| {
                let is_video = att.media_type.starts_with("video/");
                view! {
                    <div>
                        {if is_video {
                            view! {
                                <MediaVideo
                                    url=att.url
                                    preview_url=att.preview_url
                                />
                            }.into_any()
                        } else {
                            view! {
                                <MediaImage
                                    url=att.url
                                    alt=att.alt
                                    preview_url=att.preview_url
                                    is_sensitive=att.is_sensitive
                                />
                            }.into_any()
                        }}
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
