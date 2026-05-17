use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::{ComposeModal, Protected};
use crate::pages::{
    AdminPage, DmConversationPage, DmPage, GlobalTimelinePage, HomePage, LocalTimelinePage,
    LoginPage, NotFoundPage, NotificationsPage, ProfilePage, SearchPage, SettingsPage, SignupPage,
    StatusDetailPage,
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
                    <Route path=path!("dm") view=|| view! { <Protected><DmPage /></Protected> } />
                    <Route path=path!("dm/:conversation") view=|| view! { <Protected><DmConversationPage /></Protected> } />
                    <Route path=path!("settings") view=|| view! { <Protected><SettingsPage /></Protected> } />
                    <Route path=path!("settings/:section") view=|| view! { <Protected><SettingsPage /></Protected> } />
                    <Route path=path!("notes/:id") view=|| view! { <Protected><StatusDetailPage /></Protected> } />
                    <Route path=path!("search") view=|| view! { <Protected><SearchPage /></Protected> } />
                    <Route path=path!("login") view=LoginPage />
                    <Route path=path!("signup") view=SignupPage />
                    <Route path=path!("profile/:username") view=|| view! { <Protected><ProfilePage /></Protected> } />
                    <Route path=path!("admin") view=|| view! { <Protected><AdminPage /></Protected> } />
                </Routes>
                <ComposeModal />
            </div>
        </Router>
    }
}
