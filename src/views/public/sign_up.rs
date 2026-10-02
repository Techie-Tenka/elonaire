use detaxine_ui::components::feedback::{
    modal::modal::{BasicModal, UseCase},
    spinner::Spinner,
};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::{
    components::molecules::auth::sign_up_form::SignUpForm,
    data::context::{auth::use_auth, ui::use_ui},
};

#[component]
pub fn SignUp() -> impl IntoView {
    let auth_ctx = use_auth();
    let ui_ctx = use_ui();
    let navigate = use_navigate();

    let success_modal_is_open = RwSignal::new(false);

    // Once authenticated (either via credentials or OAuth), redirect to the
    // previously-remembered page or fall back to /dashboard.
    Effect::new(move |_| {
        if !auth_ctx.is_authenticated().get_untracked() {
            return;
        }

        if let Some(redirect_to) = ui_ctx.redirect_to.get() {
            ui_ctx.error.set(None);
            navigate(&redirect_to, Default::default());
        } else {
            navigate("/dashboard", Default::default());
        }
    });

    // SignUpForm fires this after a successful `signUp` mutation (the user
    // exists but is not yet authenticated — they must confirm their email).
    // The form handles the reset; the page just shows the success modal.
    let handle_signup_success = Callback::new(move |_| {
        success_modal_is_open.set(true);
    });

    view! {
        <Title text="Sign Up"/>

        <Show when=move || auth_ctx.is_loading.get()>
            <Spinner />
        </Show>

        <BasicModal
            title="Sign Up Successful"
            is_open=success_modal_is_open
            use_case=UseCase::Success
            disable_auto_close=false
        >
            <div class="p-[10px]">
                <p>"Your registration was successful!"</p>
                <p>"A confirmation email was sent to your inbox. Confirm your email for your account to be activated."</p>
            </div>
        </BasicModal>

        <div class="flex flex-col items-center justify-center p-8 min-h-svh">
            <h1 class="text-4xl font-bold my-4">"Create Account"</h1>

            <SignUpForm on_success=handle_signup_success />

            <div class="flex items-center justify-center mt-6 text-sm text-secondary w-full max-w-md">
                <A href="/sign-in">"Already have an account? Sign in"</A>
            </div>
        </div>
    }
    .into_any()
}
