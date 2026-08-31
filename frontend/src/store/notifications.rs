use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct NotificationStore {
    pub unread_notifications: RwSignal<u32>,
}

impl NotificationStore {
    pub fn new() -> Self {
        Self {
            unread_notifications: RwSignal::new(0),
        }
    }

    pub fn mark_notifications_read(&self) {
        self.unread_notifications.set(0);
    }
}
