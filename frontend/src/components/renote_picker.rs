use leptos::prelude::*;

use crate::models::NoteVisibility;

#[component]
pub fn RenotePicker(
    is_open: bool,
    on_select: Callback<Option<NoteVisibility>>,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || is_open>
            <div class="wf-pop" style="width:220px;">
                <button
                    class="wf-pop-item"
                    on:click=move |_| {
                        on_select.run(None);
                        on_close.run(());
                    }
                >
                    <span class="text-lg mr-2">{":globe:"}</span>
                    "リノート"
                </button>
                <button
                    class="wf-pop-item"
                    on:click=move |_| {
                        on_select.run(Some(NoteVisibility::Home));
                        on_close.run(());
                    }
                >
                    <span class="text-lg mr-2">{":house:"}</span>
                    "ホームに引用"
                </button>
                <button
                    class="wf-pop-item"
                    on:click=move |_| {
                        on_select.run(Some(NoteVisibility::Followers));
                        on_close.run(());
                    }
                >
                    <span class="text-lg mr-2">{":lock:"}</span>
                    "フォロワー限定引用"
                </button>
                <button
                    class="wf-pop-item"
                    on:click=move |_| {
                        on_select.run(Some(NoteVisibility::Specified));
                        on_close.run(());
                    }
                >
                    <span class="text-lg mr-2">{":envelope:"}</span>
                    "DMに引用"
                </button>
            </div>
        </Show>
    }
}

#[component]
pub fn VisibilityChooser(
    current: NoteVisibility,
    on_change: Callback<NoteVisibility>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);

    let select = move |v: NoteVisibility| {
        on_change.run(v);
        is_open.set(false);
    };

    let icon = move || match current {
        NoteVisibility::Public => ":globe:",
        NoteVisibility::Home => ":house:",
        NoteVisibility::Followers => ":lock:",
        NoteVisibility::Specified => ":envelope:",
    };
    let label = move || match current {
        NoteVisibility::Public => "公開",
        NoteVisibility::Home => "ホーム",
        NoteVisibility::Followers => "フォロワー",
        NoteVisibility::Specified => "指定ユーザー",
    };

    view! {
        <div class="relative">
            <button
                class="wf-btn wf-btn-ghost wf-btn-sm"
                on:click=move |_| is_open.update(|v| *v = !*v)
            >
                <span>{move || icon()}</span>
                <span class="text-xs">{move || label()}</span>
            </button>
            <Show when=move || is_open.get()>
                <div class="wf-pop" style="width:160px;">
                    {[
                        (NoteVisibility::Public, ":globe:", "公開"),
                        (NoteVisibility::Home, ":house:", "ホーム"),
                        (NoteVisibility::Followers, ":lock:", "フォロワー"),
                        (NoteVisibility::Specified, ":envelope:", "指定ユーザー"),
                    ].iter().map(|(v, icon, label)| {
                        let v = *v;
                        let is_active = move || current == v;
                        view! {
                            <button
                                class=move || if is_active() { "wf-pop-item active" } else { "wf-pop-item" }
                                on:click=move |_| select(v)
                            >
                                <span class="mr-2">{icon.to_string()}</span>
                                {label.to_string()}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </Show>
        </div>
    }
}
