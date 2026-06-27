use leptos::prelude::*;

#[component]
pub fn LoadingSpinner(
    #[prop(default = "loading".into())]
    label: String,
    #[prop(default = "md")]
    size: &'static str,
) -> impl IntoView {
    let size_class = match size {
        "xs" => "loading-xs",
        "sm" => "loading-sm",
        "lg" => "loading-lg",
        _ => "loading-md",
    };
    view! {
        <div class="flex flex-col items-center justify-center gap-2 py-8 text-base-content/50">
            <span class=format!("loading loading-spinner {}", size_class) />
            <span class="text-sm">{label}</span>
        </div>
    }
}
