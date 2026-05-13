use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::{ComposeModal, Protected};
use crate::pages::{
    DmConversationPage, DmPage, GlobalTimelinePage, HomePage, LocalTimelinePage, LoginPage,
    NotFoundPage, NotificationsPage, ProfilePage, SearchPage, SettingsPage, StatusDetailPage,
};
use crate::store::{AuthStore, ComposeStore, NotificationStore};

#[component]
pub fn App() -> impl IntoView {
    provide_context(AuthStore::new());
    provide_context(ComposeStore::new());
    provide_context(NotificationStore::new());

    view! {
        <Router>
            <div class="app-root">
                <Routes fallback=NotFoundPage>
                    <Route path=path!("") view=|| view! { <Protected><HomePage /></Protected> } />
                    <Route path=path!("local") view=|| view! { <Protected><LocalTimelinePage /></Protected> } />
                    <Route path=path!("global") view=|| view! { <Protected><GlobalTimelinePage /></Protected> } />
                    <Route path=path!("notifications") view=|| view! { <Protected><NotificationsPage /></Protected> } />
                    <Route path=path!("messages") view=|| view! { <Protected><DmPage /></Protected> } />
                    <Route path=path!("messages/:id") view=|| view! { <Protected><DmConversationPage /></Protected> } />
                    <Route path=path!("settings") view=|| view! { <Protected><SettingsPage /></Protected> } />
                    <Route path=path!("settings/:section") view=|| view! { <Protected><SettingsPage /></Protected> } />
                    <Route path=path!("notes/:id") view=StatusDetailPage />
                    <Route path=path!("search") view=SearchPage />
                    <Route path=path!("login") view=LoginPage />
                    <Route path=path!(":handle") view=ProfilePage />
                </Routes>
                <ComposeModal />
            </div>
        </Router>
    }
}
