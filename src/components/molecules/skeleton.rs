use leptos::prelude::*;

#[component]
pub fn Skeleton(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <span class=format!(
            "inline-block animate-pulse rounded-[4px] bg-gray-300/70 dark:bg-gray-700/70 {class}"
        )></span>
    }
}
