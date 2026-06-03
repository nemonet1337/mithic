use leptos::prelude::*;

use crate::models::User;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarSize {
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

impl AvatarSize {
    pub fn class_name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarAccent {
    #[default]
    None,
    Accent,
    Accent2,
    Ink,
}

impl AvatarAccent {
    fn extra_class(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Accent => " accent",
            Self::Accent2 => " accent2",
            Self::Ink => " ink",
        }
    }
}

#[component]
pub fn Avatar(
    user: User,
    #[prop(default = AvatarSize::Md)] size: AvatarSize,
    #[prop(default = AvatarAccent::None)] accent: AvatarAccent,
) -> impl IntoView {
    let initials = user.initials();
    let label = user.handle();
    let name = user.name();
    let avatar_url = user.avatar_url.clone();
    let size_cls = size.class_name();
    let class = if size_cls.is_empty() {
        format!("wf-av{} avatar-content", accent.extra_class())
    } else {
        format!("wf-av {} avatar-content{}", size_cls, accent.extra_class())
    };
    view! {
        <div class=class aria-label=label>
            {if let Some(url) = avatar_url {
                view! { <img src=url alt=name loading="lazy" /> }.into_any()
            } else {
                view! { <span>{initials}</span> }.into_any()
            }}
        </div>
    }
}
