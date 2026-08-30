use leptos::prelude::*;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;

#[component]
pub fn LoadMore(on_visible: Arc<dyn Fn() + Send + Sync>) -> impl IntoView {
    let root_ref = NodeRef::<leptos::html::Div>::new();
    let (fired, set_fired) = signal(false);
    let callback = Arc::clone(&on_visible);

    Effect::new(move |_| {
        if fired.get() {
            return;
        }
        let Some(el) = root_ref.get() else {
            return;
        };
        set_fired.set(true);

        let cb = Arc::clone(&callback);
        let handler = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
            let entry: JsValue = entries.get(0);
            if !entry.is_undefined() {
                let entry: web_sys::IntersectionObserverEntry = entry.dyn_into().unwrap();
                if entry.is_intersecting() {
                    cb();
                }
            }
        });
        let observer = match web_sys::IntersectionObserver::new(handler.as_ref().unchecked_ref()) {
            Ok(o) => o,
            Err(_) => return,
        };
        let _ = observer.observe(&el);
        handler.forget();
    });

    view! {
        <div node_ref=root_ref class="timeline-sentinel py-4 text-center" style="height:1px"></div>
    }
}
