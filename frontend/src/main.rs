mod api;
mod app;
mod components;
mod models;
mod pages;
mod store;
mod time;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
