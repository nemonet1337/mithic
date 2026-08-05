use leptos::prelude::*;

use crate::api::drive::DriveFileResponse;
use crate::components::Shell;

#[component]
pub fn DrivePage() -> impl IntoView {
    let files = RwSignal::<Vec<DriveFileResponse>>::new(Vec::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::<Option<String>>::new(None);

    let auth = expect_context::<crate::store::AuthStore>();

    let load_files = move || {
        let token = auth.token.get_untracked();
        let Some(tok) = token else {
            loading.set(false);
            return;
        };
        loading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::drive::find(&tok, None, None, Some(50)).await {
                Ok(list) => {
                    files.set(list);
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    loading.set(false);
                }
            }
        });
    };

    Effect::new(move |_| {
        load_files();
    });

    let delete_file = move |file_id: String| {
        let token = auth.token.get_untracked();
        let Some(tok) = token else { return; };
        wasm_bindgen_futures::spawn_local(async move {
            if crate::api::drive::delete(&tok, &file_id).await.is_ok() {
                files.update(|v| v.retain(|f| f.id != file_id));
            }
        });
    };

    view! {
        <Shell active="drive">
            <section class="p-4">
                <h1 class="wf-title mb-4">"ファイルマネージャー"</h1>

                <Show when=move || loading.get()>
                    <div class="flex items-center justify-center py-16">
                        <span class="wf-spinner" style="width:32px;height:32px;" />
                    </div>
                </Show>

                <Show when=move || error.get().is_some()>
                    <div class="wf-alert error">
                        <span>{move || error.get().unwrap_or_default()}</span>
                    </div>
                </Show>

                <Show when=move || !loading.get() && files.get().is_empty()>
                    <div class="wf-empty">
                        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>
                        <p class="text-sm">"ファイルがありません"</p>
                    </div>
                </Show>

                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3 mt-4">
                    <For
                        each=move || files.get()
                        key=|f| f.id.clone()
                        children=move |file: DriveFileResponse| {
                            let is_image = file.mime_type.starts_with("image/");
                            let is_video = file.mime_type.starts_with("video/");
                            let file_id = file.id.clone();
                            let file_url = file.url.clone();
                            let preview_url = file.preview_url.clone();
                            let file_name = file.name.clone();
                            let file_size = file.size;
                            view! {
                                <div class="wf-thumb group" style="display:flex;flex-direction:column;">
                                    {if is_image {
                                        view! {
                                            <div class="aspect-square overflow-hidden" style="background:var(--paper-3);">
                                                <img
                                                    src=preview_url.clone().unwrap_or(file_url.clone())
                                                    alt=file_name.clone()
                                                    class="w-full h-full object-cover cursor-pointer"
                                                    loading="lazy"
                                                    on:click=move |_| {
                                                        let _ = web_sys::window().unwrap().open_with_url_and_target(&file_url, "_blank");
                                                    }
                                                />
                                            </div>
                                        }.into_any()
                                    } else if is_video {
                                        view! {
                                            <div class="aspect-square flex items-center justify-center" style="background:var(--paper-3);">
                                                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"/></svg>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="aspect-square flex items-center justify-center" style="background:var(--paper-3);">
                                                <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>
                                            </div>
                                        }.into_any()
                                    }}
                                    <div class="p-2 text-xs">
                                        <p class="truncate font-mono opacity-70">{file_name}</p>
                                        <div class="flex justify-between items-center mt-1">
                                            <span class="wf-entry-meta">{format_size(file_size)}</span>
                                            <button
                                                class="wf-btn wf-btn-ghost wf-btn-sm opacity-0 group-hover:opacity-100"
                                                on:click=move |_| delete_file(file_id.clone())
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </section>
        </Shell>
    }
}

fn format_size(size: i64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    }
}
