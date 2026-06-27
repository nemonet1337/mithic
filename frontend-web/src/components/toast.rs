use leptos::prelude::*;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
struct ToastMessage {
    id: u32,
    text: String,
    kind: ToastKind,
}

#[derive(Clone, Copy)]
pub struct ToastStore {
    messages: RwSignal<Vec<ToastMessage>>,
    counter: RwSignal<u32>,
}

impl ToastStore {
    pub fn new() -> Self {
        Self {
            messages: RwSignal::new(Vec::new()),
            counter: RwSignal::new(0),
        }
    }

    pub fn push(&self, text: impl Into<String>, kind: ToastKind) {
        let id = self.counter.get_untracked();
        self.counter.update(|c| *c += 1);
        let msg = ToastMessage {
            id,
            text: text.into(),
            kind,
        };
        self.messages.update(|msgs| {
            if msgs.len() >= 5 {
                msgs.remove(0);
            }
            msgs.push(msg.clone());
        });
        let messages = self.messages;
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(5000).await;
            messages.update(|msgs| msgs.retain(|m| m.id != id));
        });
    }
}

#[component]
pub fn ToastContainer() -> impl IntoView {
    let store = expect_context::<ToastStore>();

    view! {
        <div class="toast toast-top toast-end z-[100]">
            <For
                each=move || store.messages.get()
                key=|msg| msg.id
                children=move |msg: ToastMessage| {
                    let kind = msg.kind;
                    let text = msg.text;
                    let alert_class = match kind {
                        ToastKind::Info => "alert-info",
                        ToastKind::Success => "alert-success",
                        ToastKind::Warning => "alert-warning",
                        ToastKind::Error => "alert-error",
                    };
                    view! {
                        <div class=format!("alert {} shadow-lg max-w-sm", alert_class)>
                            <span class="text-sm">{text}</span>
                        </div>
                    }
                }
            />
        </div>
    }
}
