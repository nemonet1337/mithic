use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::{ComposeModal, Protected, Shell};
use crate::pages::{
    AdminPage, DmConversationPage, DmPage, DrivePage, GlobalTimelinePage, HomePage,
    LocalTimelinePage, LoginPage, NotFoundPage, NotificationsPage, ProfilePage, SearchPage,
    SettingsPage, SignupPage, StatusDetailPage,
};
use crate::store::{AuthStore, ComposeStore, NotificationStore};

#[component]
pub fn App() -> impl IntoView {
    let auth = AuthStore::new();
    auth.verify_on_startup();
    provide_context(auth);
    provide_context(ComposeStore::new());
    provide_context(NotificationStore::new());

    view! {
        <Router>
            <Routes fallback=NotFoundPage>
                <Route path=path!("") view=|| view! { <Protected><Shell active="home"><HomePage /></Shell></Protected> } />
                <Route path=path!("local") view=|| view! { <Protected><Shell active="home"><LocalTimelinePage /></Shell></Protected> } />
                <Route path=path!("global") view=|| view! { <Protected><Shell active="home"><GlobalTimelinePage /></Shell></Protected> } />
                <Route path=path!("notifications") view=|| view! { <Protected><Shell active="notif"><NotificationsPage /></Shell></Protected> } />
                <Route path=path!("dm") view=|| view! { <Protected><Shell active="dm"><DmPage /></Shell></Protected> } />
                <Route path=path!("dm/:conversation") view=|| view! { <Protected><Shell active="dm"><DmConversationPage /></Shell></Protected> } />
                <Route path=path!("settings") view=|| view! { <Protected><Shell active="settings"><SettingsPage /></Shell></Protected> } />
                <Route path=path!("settings/:section") view=|| view! { <Protected><Shell active="settings"><SettingsPage /></Shell></Protected> } />
                <Route path=path!("notes/:id") view=|| view! { <Protected><Shell active="home"><StatusDetailPage /></Shell></Protected> } />
                <Route path=path!("search") view=|| view! { <Protected><Shell active="search"><SearchPage /></Shell></Protected> } />
                <Route path=path!("login") view=LoginPage />
                <Route path=path!("signup") view=SignupPage />
                <Route path=path!("profile/:username") view=|| view! { <Protected><Shell active="profile"><ProfilePage /></Shell></Protected> } />
                <Route path=path!("admin") view=|| view! { <Protected><Shell active="settings"><AdminPage /></Shell></Protected> } />
                <Route path=path!("drive") view=|| view! { <Protected><Shell active="profile"><DrivePage /></Shell></Protected> } />
            </Routes>
            <ComposeModal />
        </Router>
    }
}
