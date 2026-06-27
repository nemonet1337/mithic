use leptos::prelude::*;

#[component]
pub fn Autocomplete<T: Clone + PartialEq + 'static>(
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
            <ul class="menu menu-sm bg-base-200 rounded-box absolute z-50 mt-1 w-full shadow-lg border border-base-300">
                {move || filtered.get().into_iter().map(|item| {
                    let label = display(&item);
                    let cb = on_select;
                    let item = item.clone();
                    view! {
                        <li>
                            <button on:click=move |_| cb.call(item.clone())>
                                {label}
                            </button>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        </Show>
    }
}
