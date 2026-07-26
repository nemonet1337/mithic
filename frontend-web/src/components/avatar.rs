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
            Self::Sm => "w-8 h-8 text-xs",
            Self::Md => "w-10 h-10 text-sm",
            Self::Lg => "w-14 h-14 text-base",
            Self::Xl => "w-20 h-20 text-xl",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarAccent {
    #[default]
    None,
    Accent,
    Ink,
}

impl AvatarAccent {
    fn extra_class(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Accent => "avatar-accent",
            Self::Ink => "",
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
    let size_class = size.class_name();
    let accent_class = accent.extra_class();

    view! {
        <div class=format!("avatar {}", accent_class) aria-label=label>
            <div class=size_class>
                {if let Some(url) = avatar_url {
                    view! {
                        <img src=url alt=name loading="lazy" />
                    }.into_any()
                } else {
                    view! {
                        <span class="font-bold uppercase">{initials}</span>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
