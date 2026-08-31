use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::{ComposeModal, Protected, ToastContainer, ToastStore};
use crate::pages::{
    AdminPage, DmConversationPage, DmPage, DrivePage, GlobalTimelinePage, HomePage,
    LocalTimelinePage, LoginPage, NotFoundPage, NotificationsPage, ProfilePage, SearchPage,
    SettingsPage, SignupPage, StatusDetailPage,
};
use crate::store::{
    AuthStore, ComposeStore, NotificationStore, StreamStore, stream::connect_stream,
};

#[component]
pub fn App() -> impl IntoView {
    let auth = AuthStore::new();
    auth.verify_on_startup();
    let auth_for_stream = auth.clone();
    provide_context(auth);
    provide_context(ComposeStore::new());
    let notifications = NotificationStore::new();
    provide_context(notifications);
    provide_context(ToastStore::new());
    let stream = StreamStore::new();
    provide_context(stream);

    let last_stream_token = RwSignal::new(None::<String>);
    Effect::new(move |_| {
        if auth_for_stream.me.get().is_none() {
            return;
        }
        let Some(token) = auth_for_stream.token.get() else {
            return;
        };
        if last_stream_token.get_untracked().as_ref() == Some(&token) {
            return;
        }
        last_stream_token.set(Some(token.clone()));
        connect_stream(token, stream, notifications);
    });

    view! {
        <Router>
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
                <Route path=path!("drive") view=|| view! { <Protected><DrivePage /></Protected> } />
            </Routes>
            <ComposeModal />
            <ToastContainer />
        </Router>
    }
}
