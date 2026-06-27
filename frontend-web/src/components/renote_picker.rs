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
            <div class="absolute z-50 mt-1 p-3 bg-base-100 border border-base-300 rounded-2xl shadow-xl w-56">
                <div class="flex flex-col gap-1">
                    <button
                        class="btn btn-ghost btn-sm justify-start"
                        on:click=move |_| {
                            on_select.call(None);
                            on_close.call(());
                        }
                    >
                        <span class="text-lg mr-2">🌏</span>
                        "リノート"
                    </button>
                    <button
                        class="btn btn-ghost btn-sm justify-start"
                        on:click=move |_| {
                            on_select.call(Some(NoteVisibility::Home));
                            on_close.call(());
                        }
                    >
                        <span class="text-lg mr-2">🏠</span>
                        "ホームに引用"
                    </button>
                    <button
                        class="btn btn-ghost btn-sm justify-start"
                        on:click=move |_| {
                            on_select.call(Some(NoteVisibility::Followers));
                            on_close.call(());
                        }
                    >
                        <span class="text-lg mr-2">🔒</span>
                        "フォロワー限定引用"
                    </button>
                    <button
                        class="btn btn-ghost btn-sm justify-start"
                        on:click=move |_| {
                            on_select.call(Some(NoteVisibility::Specified));
                            on_close.call(());
                        }
                    >
                        <span class="text-lg mr-2">✉️</span>
                        "DMに引用"
                    </button>
                </div>
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
        on_change.call(v);
        is_open.set(false);
    };

    let icon = move || match current {
        NoteVisibility::Public => "🌏",
        NoteVisibility::Home => "🏠",
        NoteVisibility::Followers => "🔒",
        NoteVisibility::Specified => "✉️",
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
                class="btn btn-ghost btn-xs gap-1"
                on:click=move |_| is_open.update(|v| *v = !*v)
            >
                <span>{move || icon()}</span>
                <span class="text-xs">{move || label()}</span>
            </button>
            <Show when=move || is_open.get()>
                <div class="absolute z-50 mt-1 p-2 bg-base-100 border border-base-300 rounded-xl shadow-lg w-40">
                    {[
                        (NoteVisibility::Public, "🌏", "公開"),
                        (NoteVisibility::Home, "🏠", "ホーム"),
                        (NoteVisibility::Followers, "🔒", "フォロワー"),
                        (NoteVisibility::Specified, "✉️", "指定ユーザー"),
                    ].iter().map(|(v, icon, label)| {
                        let v = *v;
                        let is_active = move || current == v;
                        view! {
                            <button
                                class=move || format!(
                                    "btn btn-ghost btn-sm justify-start w-full {}",
                                    if is_active() { "btn-active" } else { "" }
                                )
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
