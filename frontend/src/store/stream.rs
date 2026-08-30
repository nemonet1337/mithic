use std::collections::HashSet;

use leptos::prelude::*;

use crate::models::Note;
#[cfg(target_arch = "wasm32")]
use crate::models::NoteVisibility;
use crate::store::NotificationStore;
#[cfg(target_arch = "wasm32")]
use shared::StreamEvent;

#[derive(Clone, Copy)]
pub struct StreamStore {
    pub latest_note: RwSignal<Option<Note>>,
    pub deleted_ids: RwSignal<HashSet<String>>,
}

impl StreamStore {
    pub fn new() -> Self {
        Self {
            latest_note: RwSignal::new(None),
            deleted_ids: RwSignal::new(HashSet::new()),
        }
    }

    pub fn mark_deleted(&self, id: impl Into<String>) {
        let id = id.into();
        self.deleted_ids.update(|set| {
            set.insert(id);
        });
    }

    pub fn visible(&self, notes: Vec<Note>) -> Vec<Note> {
        let deleted = self.deleted_ids.get();
        notes
            .into_iter()
            .filter(|n| !note_is_deleted(n, &deleted))
            .collect()
    }
}

fn note_is_deleted(note: &Note, deleted: &HashSet<String>) -> bool {
    deleted.contains(&note.id)
        || note
            .renote
            .as_ref()
            .is_some_and(|inner| deleted.contains(&inner.id))
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Default)]
struct SeenNoteBuffer {
    seen: HashSet<String>,
    order: std::collections::VecDeque<String>,
    capacity: usize,
}

#[cfg(target_arch = "wasm32")]
impl SeenNoteBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            seen: HashSet::new(),
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
pub fn connect_stream(token: String, stream: StreamStore, notif_store: NotificationStore) {
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

    let onmessage =
        Closure::<dyn FnMut(MessageEvent)>::wrap(Box::new(move |event: MessageEvent| {
            if let Some(text) = event.data().as_string() {
                match serde_json::from_str::<StreamEvent>(&text) {
                    Ok(StreamEvent::Note(note)) => {
                        let note = *note;
                        if note.visibility == NoteVisibility::Specified {
                            return;
                        }
                        let id = note.id.clone();
                        let is_new = seen.try_update(|buf| buf.insert(id)).unwrap_or(false);
                        if is_new {
                            stream.latest_note.set(Some(note));
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
                    Ok(StreamEvent::NoteDeleted { id }) => {
                        stream.mark_deleted(id);
                    }
                    Err(_) => {}
                }
            }
        }));

    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let token_for_close = token.clone();
    let stream_for_close = stream;
    let store_for_close = notif_store;
    let onclose = Closure::<dyn FnMut(CloseEvent)>::wrap(Box::new(move |_| {
        let tok = token_for_close.clone();
        let stream = stream_for_close;
        let store = store_for_close;
        Timeout::new(3_000, move || {
            connect_stream(tok, stream, store);
        })
        .forget();
    }));
    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
    onclose.forget();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn connect_stream(_token: String, _stream: StreamStore, _notif_store: NotificationStore) {}
