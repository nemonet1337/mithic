use leptos::prelude::*;

#[component]
pub fn ConfirmDialog(
    #[prop(into)] is_open: Signal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(into, optional)] body: Signal<String>,
    #[prop(into, optional)] preview_meta: Signal<String>,
    #[prop(into, optional)] preview: Signal<String>,
    #[prop(into)] confirm_label: Signal<String>,
    #[prop(optional, into)] danger: Signal<bool>,
    on_confirm: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    let confirm_class = move || {
        if danger.get() {
            "wf-btn wf-btn-danger"
        } else {
            "wf-btn wf-btn-primary"
        }
    };

    view! {
        <Show when=move || is_open.get()>
            <div class="wf-overlay" on:click=move |_| on_close.run(())>
                <div
                    class="wf-modal wf-confirm"
                    style="max-width:420px;"
                    on:click=move |ev| ev.stop_propagation()
                >
                    <div class="wf-modal-body" style="display:flex;flex-direction:column;gap:12px;">
                        {move || danger.get().then(|| view! {
                            <span class="wf-pill on" style="align-self:flex-start;">"[ 注意 ]"</span>
                        })}
                        <h2 class="wf-modal-title">{move || title.get()}</h2>
                        <p class="text-sm text-ink-soft leading-relaxed">{move || body.get()}</p>
                        <div class="wf-card" style="border-left:3px solid var(--accent);">
                            <span class="wf-entry-meta">{move || preview_meta.get()}</span>
                            <p class="text-sm italic mt-1 text-ink-soft">{move || preview.get()}</p>
                        </div>
                        <div class="flex justify-end gap-2 mt-2">
                            <button class="wf-btn wf-btn-ghost" on:click=move |_| on_close.run(())>
                                "キャンセル"
                            </button>
                            <button
                                class=confirm_class
                                on:click=move |_| {
                                    on_confirm.run(());
                                    on_close.run(());
                                }
                            >
                                {move || confirm_label.get()}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
