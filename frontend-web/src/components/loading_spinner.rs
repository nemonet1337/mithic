use leptos::prelude::*;

#[component]
pub fn LoadingSpinner(
    #[prop(default = "loading".into())]
    label: String,
    #[prop(default = "md")]
    size: &'static str,
) -> impl IntoView {
    let px = match size {
        "xs" => "14px",
        "sm" => "18px",
        "lg" => "32px",
        _ => "22px",
    };
    view! {
        <div class="flex flex-col items-center justify-center gap-2 py-8 wf-entry-meta">
            <span class="wf-spinner" style=format!("width:{px};height:{px};") />
            <span class="text-sm">{label}</span>
        </div>
    }
}
