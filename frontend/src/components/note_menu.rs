use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteMenuAction {
    CopyLink,
    Delete,
    Report,
    Mute,
    Block,
    Pin,
}

#[component]
pub fn NoteMenu(
    #[prop(into)] is_open: Signal<bool>,
    #[prop(into)] is_own_note: Signal<bool>,
    on_action: Callback<NoteMenuAction>,
    on_close: Callback<()>,
) -> impl IntoView {
    let do_action = move |action: NoteMenuAction| {
        on_action.run(action);
        on_close.run(());
    };

    view! {
        <Show when=move || is_open.get()>
            <div class="wf-pop" style="right:0;width:200px;">
                <button
                    class="wf-pop-item"
                    on:click=move |_| do_action(NoteMenuAction::CopyLink)
                >
                    "リンクをコピー"
                </button>
                <button
                    class="wf-pop-item"
                    on:click=move |_| do_action(NoteMenuAction::Pin)
                >
                    "ピン留め"
                </button>
                <button
                    class="wf-pop-item"
                    on:click=move |_| do_action(NoteMenuAction::Mute)
                >
                    "ミュート"
                </button>
                <button
                    class="wf-pop-item"
                    on:click=move |_| do_action(NoteMenuAction::Block)
                >
                    "ブロック"
                </button>
                <button
                    class="wf-pop-item"
                    on:click=move |_| do_action(NoteMenuAction::Report)
                >
                    "通報"
                </button>
                <Show when=move || is_own_note.get()>
                    <hr class="wf-spine-rule" style="margin:4px 0;" />
                    <button
                        class="wf-pop-item danger"
                        on:click=move |_| do_action(NoteMenuAction::Delete)
                    >
                        "削除"
                    </button>
                </Show>
            </div>
        </Show>
    }
}
