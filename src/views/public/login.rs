use detaxine_ui::components::feedback::spinner::Spinner;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::components::molecules::auth::sign_in_form::SignInForm;
use crate::data::context::{auth::use_auth, ui::use_ui};
use crate::views::public::error_handler::ErrorHandler;

#[component]
pub fn SignIn() -> impl IntoView {
    let auth_ctx = use_auth();
    let ui_ctx = use_ui();
    let navigate = use_navigate();

    let handle_success = Callback::new(move |_| {
        if let Some(redirect_to) = ui_ctx.redirect_to.get() {
            ui_ctx.error.set(None);
            navigate(&redirect_to, Default::default());
        } else {
            navigate("/dashboard", Default::default());
        }
    });

    view! {
        <Title text="Sign In"/>
        <Show when=move || auth_ctx.is_loading.get()>
            <Spinner />
        </Show>
        <ErrorHandler />
        <div class="flex flex-col items-center justify-center p-8 min-h-svh">
            <A href="/" attr:class="flex items-center h-[50px]">
                <img src="https://techietenka.com/api/files/view/default/Techie Tenka-01.png" class="h-full w-auto object-cover" alt="Logo" />
            </A>
            <h1 class="text-4xl font-bold my-4">"Sign In"</h1>
            <SignInForm on_success=handle_success />
        </div>
    }
    .into_any()
}
