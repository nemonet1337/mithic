use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;

const STORAGE_KEY: &str = "mithic.deck.columns";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnKind {
    Home,
    Local,
    Global,
    Notifications,
}

impl ColumnKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Local => "local",
            Self::Global => "global",
            Self::Notifications => "notifications",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Home => "ホーム",
            Self::Local => "ローカル",
            Self::Global => "グローバル",
            Self::Notifications => "通知",
        }
    }

    pub fn path(self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Local => "/local",
            Self::Global => "/global",
            Self::Notifications => "/notifications",
        }
    }

    pub fn api_kind(self) -> Option<&'static str> {
        match self {
            Self::Home => Some("home"),
            Self::Local => Some("local"),
            Self::Global => Some("global"),
            Self::Notifications => None,
        }
    }

    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "home" => Some(Self::Home),
            "local" => Some(Self::Local),
            "global" => Some(Self::Global),
            "notifications" => Some(Self::Notifications),
            _ => None,
        }
    }

    pub fn from_path(path: &str) -> Option<Self> {
        match path.trim_end_matches('/') {
            "" | "/" => Some(Self::Home),
            "/local" => Some(Self::Local),
            "/global" => Some(Self::Global),
            "/notifications" => Some(Self::Notifications),
            _ => None,
        }
    }

    pub fn all() -> [Self; 4] {
        [Self::Home, Self::Local, Self::Global, Self::Notifications]
    }
}

fn default_columns() -> Vec<ColumnKind> {
    vec![ColumnKind::Home, ColumnKind::Local, ColumnKind::Global]
}

fn load_columns() -> Vec<ColumnKind> {
    let Ok(ids) = LocalStorage::get::<Vec<String>>(STORAGE_KEY) else {
        return default_columns();
    };
    let mut seen = std::collections::HashSet::new();
    let cols: Vec<_> = ids
        .into_iter()
        .filter_map(|s| ColumnKind::from_id(&s))
        .filter(|c| seen.insert(*c))
        .collect();
    if cols.is_empty() {
        default_columns()
    } else {
        cols
    }
}

fn persist(cols: &[ColumnKind]) {
    let ids: Vec<&str> = cols.iter().map(|c| c.id()).collect();
    let _ = LocalStorage::set(STORAGE_KEY, ids);
}

#[derive(Clone, Copy)]
pub struct DeckStore {
    pub columns: RwSignal<Vec<ColumnKind>>,
}

impl DeckStore {
    pub fn new() -> Self {
        Self {
            columns: RwSignal::new(load_columns()),
        }
    }

    fn commit(&self, cols: Vec<ColumnKind>) {
        persist(&cols);
        self.columns.set(cols);
    }

    pub fn add(&self, kind: ColumnKind) {
        let mut cols = self.columns.get_untracked();
        if !cols.contains(&kind) {
            cols.push(kind);
            self.commit(cols);
        }
    }

    pub fn remove(&self, kind: ColumnKind) {
        let mut cols = self.columns.get_untracked();
        if cols.len() <= 1 {
            return;
        }
        cols.retain(|c| *c != kind);
        self.commit(cols);
    }

    pub fn move_left(&self, kind: ColumnKind) {
        let mut cols = self.columns.get_untracked();
        if let Some(i) = cols.iter().position(|c| *c == kind)
            && i > 0
        {
            cols.swap(i, i - 1);
            self.commit(cols);
        }
    }

    pub fn move_right(&self, kind: ColumnKind) {
        let mut cols = self.columns.get_untracked();
        if let Some(i) = cols.iter().position(|c| *c == kind)
            && i + 1 < cols.len()
        {
            cols.swap(i, i + 1);
            self.commit(cols);
        }
    }

    pub fn visible_with_focus(&self, focus: Option<ColumnKind>) -> Vec<ColumnKind> {
        let mut cols = self.columns.get();
        if let Some(f) = focus
            && !cols.contains(&f)
        {
            cols.push(f);
        }
        cols
    }

    pub fn missing(&self) -> Vec<ColumnKind> {
        let cols = self.columns.get();
        ColumnKind::all()
            .into_iter()
            .filter(|c| !cols.contains(c))
            .collect()
    }
}
