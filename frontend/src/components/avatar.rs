use leptos::prelude::*;

use shared::User;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarSize {
    Sm,
    #[default]
    Md,
    Xl,
}

impl AvatarSize {
    pub fn class_name(self) -> &'static str {
        match self {
            Self::Sm => "w-8 h-8 text-xs",
            Self::Md => "w-10 h-10 text-sm",
            Self::Xl => "w-20 h-20 text-xl",
        }
    }
}

#[component]
pub fn Avatar(
    user: User,
    #[prop(default = AvatarSize::Md)] size: AvatarSize,
    #[prop(default = false)] accent: bool,
) -> impl IntoView {
    let initials = user.initials();
    let label = user.handle();
    let name = user.name();
    let avatar_url = user.avatar_url.clone();
    let size_class = size.class_name();
    let accent_class = if accent { "avatar-accent" } else { "" };

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
