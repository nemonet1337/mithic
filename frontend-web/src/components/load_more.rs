use leptos::prelude::*;

#[component]
pub fn LoadMore<F>(on_visible: F) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
    let root_ref = NodeRef::<leptos::html::Div>::new();
    let (fired, set_fired) = signal(false);
    let callback = on_visible.clone();

    Effect::new(move |_| {
        if fired.get() {
            return;
        }
        let Some(el) = root_ref.get() else {
            return;
        };
        set_fired.set(true);

        let cb = callback.clone();
        let observer = match web_sys::IntersectionObserver::new(
            &web_sys::IntersectionObserverInit::new(),
        ) {
            Ok(o) => o,
            Err(_) => return,
        };

        let handler = Closure::<dyn FnMut(js_sys::Array)>::new(
            move |entries: js_sys::Array| {
                if let Some(e) = entries.get(0) {
                    let rect: web_sys::IntersectionObserverEntry = e.into();
                    if rect.is_intersecting() {
                        cb();
                    }
                }
            },
        );
        let _ = observer.observe_with_node(&el);
        handler.forget();
    });

    view! {
        <div node_ref=root_ref class="timeline-sentinel py-4 text-center" style="height:1px"></div>
    }
}
