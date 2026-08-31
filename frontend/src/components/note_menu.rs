use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NoteMenuAction {
    CopyLink,
    Pin,
    Mute,
    Block,
    Report,
    Delete,
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
            <div class="wf-pop" style="right:0;width:220px;">
                <div class="wf-entry-meta px-2.5 pt-1">"[ この投稿 ]"</div>
                <button class="wf-pop-item" on:click=move |_| do_action(NoteMenuAction::CopyLink)>
                    "URLをコピー"
                </button>
                <button class="wf-pop-item" on:click=move |_| do_action(NoteMenuAction::Pin)>
                    "プロフィールに固定"
                </button>
                <hr class="wf-rule" />
                <div class="wf-entry-meta px-2.5 pt-1">"[ このユーザー ]"</div>
                <button class="wf-pop-item" on:click=move |_| do_action(NoteMenuAction::Mute)>
                    "ミュート"
                </button>
                <button class="wf-pop-item" on:click=move |_| do_action(NoteMenuAction::Block)>
                    "ブロック"
                </button>
                <hr class="wf-rule" />
                <button class="wf-pop-item danger" on:click=move |_| do_action(NoteMenuAction::Report)>
                    "通報する"
                </button>
                <Show when=move || is_own_note.get()>
                    <button class="wf-pop-item danger" on:click=move |_| do_action(NoteMenuAction::Delete)>
                        "削除"
                    </button>
                </Show>
            </div>
        </Show>
    }
}
