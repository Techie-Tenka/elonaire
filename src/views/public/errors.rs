use detaxine_ui::components::actions::button::BasicButton;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::data::context::ui::use_ui;

#[component]
pub fn NotFound() -> impl IntoView {
    let navigate = use_navigate();
    let navigate_404 = navigate.clone();
    let ui_ctx = use_ui();

    view! {
        <div class="min-h-svh bg-contrast-white dark:bg-navy flex items-center justify-center px-4">
            <div class="flex flex-col items-center text-center gap-6 max-w-md">
                <span class="text-6xl font-bold text-danger">404</span>
                <div class="flex flex-col gap-2">
                    <h1 class="text-xl font-semibold text-gray dark:text-contrast-white">
                        "Page not found"
                    </h1>
                    <p class="text-sm text-gray/60 dark:text-mid-gray leading-relaxed">
                        "The page you're looking for doesn't exist or has been moved."
                    </p>
                </div>
                <div class="flex gap-3">
                    <BasicButton
                        style_ext="text-sm px-4 py-2 rounded-[5px] border border-gray/20 dark:border-mid-gray/30 text-gray dark:text-mid-gray hover:bg-gray/5 dark:hover:bg-mid-gray/10 transition-colors"
                        on:click=move |_| {
                            if let Some(route) = ui_ctx.redirect_to.get() {
                                navigate_404(&route, Default::default());
                            }
                        }
                    >
                        "Go back"
                    </BasicButton>
                    <BasicButton
                        style_ext="text-sm px-4 py-2 rounded-[5px] bg-primary text-contrast-white hover:opacity-90 transition-opacity font-medium"
                        on:click=move |_| navigate("/", Default::default())
                    >
                        "Go home"
                    </BasicButton>
                </div>
            </div>
        </div>
    }.into_any()
}

#[component]
pub fn InternalServerError() -> impl IntoView {
    let navigate = use_navigate();
    let navigate_500 = navigate.clone();
    let ui_ctx = use_ui();

    view! {
        <div class="min-h-svh bg-contrast-white dark:bg-navy flex items-center justify-center px-4">
            <div class="flex flex-col items-center text-center gap-6 max-w-md">
                <span class="text-6xl font-bold text-danger">500</span>
                <div class="flex flex-col gap-2">
                    <h1 class="text-xl font-semibold text-gray dark:text-contrast-white">
                        "Something went wrong"
                    </h1>
                    <p class="text-sm text-gray/60 dark:text-mid-gray leading-relaxed">
                        "An unexpected error occurred on our end. Please try again in a moment."
                    </p>
                </div>
                <div class="flex gap-3">
                    <BasicButton
                        style_ext="text-sm px-4 py-2 rounded-[5px] border border-gray/20 dark:border-mid-gray/30 text-gray dark:text-mid-gray hover:bg-gray/5 dark:hover:bg-mid-gray/10 transition-colors"
                        on:click=move |_| navigate("/", Default::default())
                    >
                        "Go home"
                    </BasicButton>
                    <BasicButton
                        style_ext="text-sm px-4 py-2 rounded-[5px] bg-primary text-contrast-white hover:opacity-90 transition-opacity font-medium"
                        on:click=move |_| {
                            if let Some(route) = ui_ctx.redirect_to.get() {
                                navigate_500(&route, Default::default());
                            }
                        }
                    >
                        "Try again"
                    </BasicButton>
                </div>
            </div>
        </div>
    }.into_any()
}
