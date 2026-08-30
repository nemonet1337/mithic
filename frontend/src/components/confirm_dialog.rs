use leptos::prelude::*;

#[component]
pub fn ConfirmDialog(
    #[prop(into)] is_open: Signal<bool>,
    #[prop(into)] title: String,
    #[prop(into, optional)] body: String,
    #[prop(into, optional)] preview_meta: String,
    #[prop(into, optional)] preview: String,
    #[prop(into)] confirm_label: String,
    #[prop(optional)] danger: bool,
    on_confirm: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    let confirm_class = if danger {
        "wf-btn wf-btn-danger"
    } else {
        "wf-btn wf-btn-primary"
    };

    let title = RwSignal::new(title);
    let body = RwSignal::new(body);
    let preview = RwSignal::new(preview);
    let preview_meta = RwSignal::new(preview_meta);
    let confirm_label = RwSignal::new(confirm_label);
    let danger_mark = danger;
    view! {
        <Show when=move || is_open.get()>
            <div
                class="wf-overlay"
                on:click=move |_| on_close.run(())
            >
                <div
                    class="wf-modal wf-confirm"
                    style="max-width:420px;"
                    on:click=move |ev| ev.stop_propagation()
                >
                    <div class="wf-modal-body" style="display:flex;flex-direction:column;gap:12px;">
                        {danger_mark.then(|| view! { <span class="wf-pill on" style="align-self:flex-start;">"[ 注意 ]"</span> })}
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
