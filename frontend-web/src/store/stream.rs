use leptos::prelude::*;
use serde::Deserialize;

use crate::models::{Note, Notification};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StreamEvent {
    Note { note: Note },
    Notification { notification: Notification },
}

#[cfg(target_arch = "wasm32")]
pub fn connect_stream(
    token: String,
    notes: RwSignal<Vec<Note>>,
    unread_notifications: RwSignal<u32>,
) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;
    use web_sys::{MessageEvent, WebSocket};

    let Some(window) = web_sys::window() else {
        return;
    };
    let location = window.location();
    let host = location.host().unwrap_or_else(|_| "localhost:3000".into());
    let protocol = match location.protocol().ok().as_deref() {
        Some("https:") => "wss",
        _ => "ws",
    };
    let url = format!("{protocol}://{host}/ws?token={token}");

    let Ok(ws) = WebSocket::new(&url) else {
        web_sys::console::warn_1(&"failed to open Mithic websocket".into());
        return;
    };

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::wrap(Box::new(move |event| {
        if let Some(text) = event.data().as_string() {
            match serde_json::from_str::<StreamEvent>(&text) {
                Ok(StreamEvent::Note { note }) => notes.update(|items| items.insert(0, note)),
                Ok(StreamEvent::Notification { .. }) => {
                    unread_notifications.update(|count| *count = count.saturating_add(1));
                }
                Err(error) => web_sys::console::warn_1(
                    &format!("ignored malformed stream event: {error}").into(),
                ),
            }
        }
    }));

    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn connect_stream(
    _token: String,
    _notes: RwSignal<Vec<Note>>,
    _unread_notifications: RwSignal<u32>,
) {
}
