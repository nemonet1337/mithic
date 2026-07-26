use leptos::prelude::*;
use serde::Deserialize;

use crate::models::{Note, Notification};
#[cfg(target_arch = "wasm32")]
use crate::models::NoteVisibility;
use crate::store::NotificationStore;

/// サーバー `shared::StreamEvent` と同じワイヤフォーマット
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "body", rename_all = "camelCase")]
#[allow(dead_code)]
pub enum StreamEvent {
    Note(Box<Note>),
    Notification(Box<Notification>),
}

#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct SeenNoteBuffer {
    seen: std::collections::HashSet<String>,
    order: std::collections::VecDeque<String>,
    capacity: usize,
}

#[allow(dead_code)]
impl SeenNoteBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            seen: std::collections::HashSet::new(),
            order: std::collections::VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn insert(&mut self, id: String) -> bool {
        if self.seen.contains(&id) {
            return false;
        }
        if self.order.len() >= self.capacity {
            if let Some(old) = self.order.pop_front() {
                self.seen.remove(&old);
            }
        }
        self.order.push_back(id.clone());
        self.seen.insert(id);
        true
    }
}

#[cfg(target_arch = "wasm32")]
pub fn connect_stream(
    token: String,
    notes: RwSignal<Vec<Note>>,
    notif_store: NotificationStore,
) {
    use gloo_timers::callback::Timeout;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;
    use web_sys::{CloseEvent, MessageEvent, WebSocket};

    let Some(window) = web_sys::window() else {
        return;
    };
    let location = window.location();
    let host = location.host().unwrap_or_else(|_| "localhost:3000".into());
    let protocol = match location.protocol().ok().as_deref() {
        Some("https:") => "wss",
        _ => "ws",
    };
    let url = format!("{protocol}://{host}/api/v1/streaming?token={token}");

    let Ok(ws) = WebSocket::new(&url) else {
        web_sys::console::warn_1(&"failed to open Mithic websocket".into());
        return;
    };

    let seen = RwSignal::new(SeenNoteBuffer::new(200));

    let ws_for_open = ws.clone();
    let onopen = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        // id 省略可 (サーバー側で channel 名を id に使う)
        let _ = ws_for_open.send_with_str(r#"{"type":"connect","channel":"homeTimeline"}"#);
        let _ = ws_for_open.send_with_str(r#"{"type":"connect","channel":"notifications"}"#);
        let _ = ws_for_open.send_with_str(r#"{"type":"connect","channel":"mainChannel"}"#);
    }));
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::wrap(Box::new(move |event: MessageEvent| {
        if let Some(text) = event.data().as_string() {
            match serde_json::from_str::<StreamEvent>(&text) {
                Ok(StreamEvent::Note(note)) => {
                    let note = *note;
                    if note.visibility == NoteVisibility::Specified {
                        return;
                    }
                    let id = note.id.clone();
                    if seen.get_untracked().insert(id) {
                        notes.update(|items| items.insert(0, note));
                    }
                }
                Ok(StreamEvent::Notification(notification)) => {
                    let notification = *notification;
                    let is_dm = notification
                        .note
                        .as_ref()
                        .map(|n| n.visibility == NoteVisibility::Specified)
                        .unwrap_or(false);
                    if is_dm {
                        notif_store
                            .unread_messages
                            .update(|c| *c = c.saturating_add(1));
                    } else {
                        notif_store
                            .unread_notifications
                            .update(|c| *c = c.saturating_add(1));
                    }
                }
                Err(_) => {
                    // connected / channel 等は無視
                }
            }
        }
    }));

    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let token_for_close = token.clone();
    let notes_for_close = notes;
    let store_for_close = notif_store;
    let onclose = Closure::<dyn FnMut(CloseEvent)>::wrap(Box::new(move |_| {
        let tok = token_for_close.clone();
        let notes = notes_for_close;
        let store = store_for_close;
        Timeout::new(3_000, move || {
            connect_stream(tok, notes, store);
        })
        .forget();
    }));
    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
    onclose.forget();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn connect_stream(
    _token: String,
    _notes: RwSignal<Vec<Note>>,
    _notif_store: NotificationStore,
) {
}
