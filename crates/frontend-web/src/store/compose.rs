use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;

use crate::models::NoteVisibility;

const DRAFT_KEY: &str = "mithic.compose.draft";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeVariant {
    CenteredModal,
    InlineTop,
    FullscreenWriting,
}

#[derive(Clone, Copy)]
pub struct ComposeStore {
    pub is_open: RwSignal<bool>,
    pub draft: RwSignal<String>,
    pub cw: RwSignal<String>,
    pub visibility: RwSignal<NoteVisibility>,
    pub nsfw: RwSignal<bool>,
    pub variant: RwSignal<ComposeVariant>,
}

impl ComposeStore {
    pub fn new() -> Self {
        Self {
            is_open: RwSignal::new(false),
            draft: RwSignal::new(LocalStorage::get(DRAFT_KEY).unwrap_or_default()),
            cw: RwSignal::new(String::new()),
            visibility: RwSignal::new(NoteVisibility::Public),
            nsfw: RwSignal::new(false),
            variant: RwSignal::new(ComposeVariant::CenteredModal),
        }
    }

    pub fn open(&self) {
        self.is_open.set(true);
    }

    pub fn close(&self) {
        self.is_open.set(false);
    }

    pub fn save_draft(&self) {
        if let Err(error) = LocalStorage::set(DRAFT_KEY, self.draft.get()) {
            web_sys::console::warn_1(&format!("failed to save compose draft: {error:?}").into());
        }
    }

    pub fn clear(&self) {
        self.draft.set(String::new());
        self.cw.set(String::new());
        self.nsfw.set(false);
        LocalStorage::delete(DRAFT_KEY);
    }
}
