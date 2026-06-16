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
            Self::Sm => "w-8 h-8",
            Self::Md => "w-10 h-10",
            Self::Lg => "w-14 h-14",
            Self::Xl => "w-20 h-20",
        }
    }
    pub fn text_size(self) -> &'static str {
        match self {
            Self::Sm => "text-xs",
            Self::Md => "text-sm",
            Self::Lg => "text-base",
            Self::Xl => "text-xl",
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
    fn bg_class(self) -> &'static str {
        match self {
            Self::None => "bg-primary/20 text-primary",
            Self::Accent => "bg-primary text-primary-content",
            Self::Accent2 => "bg-secondary text-secondary-content",
            Self::Ink => "bg-base-content text-base-100",
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
    let text_class = size.text_size();
    let accent_class = accent.bg_class();

    view! {
        <div class="avatar" aria-label=label>
            <div class=format!(
                "{} rounded-full ring-2 ring-base-300 ring-offset-base-100 ring-offset-1",
                size_class
            )>
                {if let Some(url) = avatar_url {
                    view! {
                        <img src=url alt=name loading="lazy" class="object-cover" />
                    }.into_any()
                } else {
                    view! {
                        <div class=format!(
                            "placeholder flex items-center justify-center w-full h-full rounded-full {} {}",
                            accent_class, text_class
                        )>
                            <span class="font-bold uppercase">{initials}</span>
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
