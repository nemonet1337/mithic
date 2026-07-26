use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteMenuAction {
    CopyLink,
    Delete,
    Report,
    Mute,
    Block,
    Pin,
    Unpin,
}

#[component]
pub fn NoteMenu(
    is_open: bool,
    is_own_note: bool,
    on_action: Callback<NoteMenuAction>,
    on_close: Callback<()>,
) -> impl IntoView {
    let do_action = move |action: NoteMenuAction| {
        on_action.run(action);
        on_close.run(());
    };

    view! {
        <Show when=move || is_open>
            <div class="wf-pop" style="right:0;width:200px;">
                <button
                    class="wf-pop-item"
                    on:click=move |_| do_action(NoteMenuAction::CopyLink)
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
                    "リンクをコピー"
                </button>
                <button
                    class="wf-pop-item"
                    on:click=move |_| do_action(NoteMenuAction::Pin)
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="17" x2="12" y2="22"/><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"/></svg>
                    "ピン留め"
                </button>
                <button
                    class="wf-pop-item"
                    on:click=move |_| do_action(NoteMenuAction::Mute)
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="1" y1="1" x2="23" y2="23"/><path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6"/><path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23"/><line x1="12" y1="19" x2="12" y2="23"/></svg>
                    "ミュート"
                </button>
                <button
                    class="wf-pop-item"
                    on:click=move |_| do_action(NoteMenuAction::Block)
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
                    "ブロック"
                </button>
                <button
                    class="wf-pop-item"
                    on:click=move |_| do_action(NoteMenuAction::Report)
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"/><line x1="4" y1="22" x2="4" y2="15"/></svg>
                    "通報"
                </button>
                {if is_own_note {
                    view! {
                        <hr class="wf-spine-rule" style="margin:4px 0;" />
                        <button
                            class="wf-pop-item danger"
                            on:click=move |_| do_action(NoteMenuAction::Delete)
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                            "削除"
                        </button>
                    }.into_any()
                } else {
                    ().into_any()
                }}
            </div>
        </Show>
    }
}
