use leptos::prelude::*;

#[component]
pub fn Autocomplete<T: Clone + PartialEq + Send + Sync + 'static>(
    items: Vec<T>,
    query: Signal<String>,
    on_select: Callback<T>,
    display: fn(&T) -> String,
    #[prop(default = 5usize)]
    max_results: usize,
) -> impl IntoView {
    let filtered = Memo::new(move |_| {
        let q = query.get().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        items
            .iter()
            .filter(|item| display(item).to_lowercase().contains(&q))
            .take(max_results)
            .cloned()
            .collect::<Vec<_>>()
    });

    view! {
        <Show when=move || !filtered.get().is_empty()>
            <ul class="wf-pop" style="position:absolute;z-index:50;margin-top:4px;width:100%;list-style:none;padding:6px;">
                {move || filtered.get().into_iter().map(|item| {
                    let label = display(&item);
                    let cb = on_select;
                    let item = item.clone();
                    view! {
                        <li>
                            <button class="wf-pop-item" on:click=move |_| cb.run(item.clone())>
                                {label}
                            </button>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        </Show>
    }
}
