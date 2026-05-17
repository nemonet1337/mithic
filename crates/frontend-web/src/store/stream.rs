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
    let url = format!("{protocol}://{host}/api/streaming?token={token}");

    let Ok(ws) = WebSocket::new(&url) else {
        web_sys::console::warn_1(&"failed to open Mithic websocket".into());
        return;
    };

    // 接続確立後にチャンネル購読メッセージを送信
    let ws_for_open = ws.clone();
    let onopen = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        let _ = ws_for_open.send_with_str(r#"{"type":"connect","channel":"homeTimeline"}"#);
        let _ = ws_for_open.send_with_str(r#"{"type":"connect","channel":"notifications"}"#);
    }));
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();

    // メッセージ受信
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

    // 切断時に指数バックオフで再接続 (簡易版: 3秒固定)
    let token_for_close = token.clone();
    let onclose = Closure::<dyn FnMut(CloseEvent)>::wrap(Box::new(move |_| {
        let tok = token_for_close.clone();
        gloo_timers::callback::Timeout::new(3_000, move || {
            connect_stream(tok, notes, unread_notifications);
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
    _unread_notifications: RwSignal<u32>,
) {
}
