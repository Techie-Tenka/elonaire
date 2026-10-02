use detaxine_ui::{
    components::{
        actions::button::{BasicButton, ButtonType},
        forms::{
            input::{InputField, InputFieldType},
            reactive_form::ReactiveForm,
        },
    },
    utils::forms::{FormDeserializeOptions, deserialize_form_with_options},
};
use icondata::{AiGithubOutlined, AiGoogleOutlined};
use leptos::ev;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::hooks::use_query;
use web_sys::{HtmlFormElement, window};

use crate::data::{
    context::{auth::use_auth, ui::use_ui},
    models::{
        general::acl::{AuthCode, OauthClientName},
        graphql::acl::UserLoginsInput,
    },
};

/// Credentials + social sign-in form. Used by the `/sign-in` page and the
/// `AuthModal`. Fires `on_success` once the session is established.
///
/// `on_switch_to_sign_up` is optional — pages that link to `/sign-up` via a
/// route can ignore it; the modal passes it to swap tabs in place.
#[component]
pub fn SignInForm(
    #[prop(optional)] on_success: Option<Callback<()>>,
    #[prop(optional)] on_switch_to_sign_up: Option<Callback<()>>,
) -> impl IntoView {
    let login_form_ref = NodeRef::new();
    let auth_ctx = use_auth();
    let ui_ctx = use_ui();
    let (form_is_valid, set_form_is_valid) = signal(false);
    let submit_is_disabled = Memo::new(move |_| !form_is_valid.get() || auth_ctx.is_loading.get());

    // OAuth callback: pick up `auth_code` from the query string and hand it
    // to the context. Safe to keep mounted in a modal — if there's no
    // `auth_code` in the URL, this is a no-op.
    let query = use_query::<AuthCode>();
    Effect::new(move |_| {
        if let Some(auth_code) = query
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.auth_code.clone())
        {
            auth_ctx.handle_oauth_callback(auth_code);
        }
    });

    // React to the social-sign-in URL being ready.
    Effect::new(move |_| {
        if let Some(url) = auth_ctx.oauth_redirect_url.get() {
            if let Some(window) = window() {
                let _ = window.open_with_url_and_target(&url, "_self");
            }
        }
    });

    // On successful auth, reset the form and hand off. Clear any pending
    // redirect target — it's the caller's job to decide where to go next.
    Effect::new(move |_| {
        if !auth_ctx.is_authenticated().get() {
            return;
        }

        if let Some(form) = login_form_ref
            .get_untracked()
            .and_then(|el: HtmlFormElement| el.dyn_into::<HtmlFormElement>().ok())
        {
            form.reset();
            set_form_is_valid.set(false);
        }

        ui_ctx.error.set(None);

        if let Some(cb) = on_success {
            cb.run(());
        }
    });

    let onsocial_sign_in = move |client: OauthClientName| {
        Callback::new(move |_e: ev::MouseEvent| {
            auth_ctx.sign_in_with_oauth(client.clone());
        })
    };

    let handle_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        ev.stop_propagation();

        let Some(form) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        else {
            return;
        };

        set_form_is_valid.set(form.check_validity());

        if !form_is_valid.get_untracked() || ev.submitter().is_none() {
            return;
        }

        let Some(user_logins) = deserialize_form_with_options::<UserLoginsInput>(
            &login_form_ref,
            &FormDeserializeOptions {
                deserialize_bool: true,
                ..Default::default()
            },
        ) else {
            return;
        };

        auth_ctx.sign_in_with_credentials(user_logins);
    };

    view! {
        <div class="w-full max-w-md flex flex-col items-center gap-2 my-4">
            <BasicButton
                button_text="Continue with Google"
                style_ext="border-[1px] border-danger hover:bg-danger transition-all duration-300 ease-in-out hover:shadow-md hover:-translate-y-1 hover:z-10 hover:text-contrast-white text-danger w-full"
                onclick=onsocial_sign_in(OauthClientName::Google)
                icon=Some(AiGoogleOutlined)
                icon_before=true
            />
            <BasicButton
                button_text="Continue with GitHub"
                style_ext="border-[1px] border-gray hover:bg-gray transition-all duration-300 ease-in-out hover:shadow-md hover:-translate-y-1 hover:z-10 hover:text-contrast-white w-full"
                onclick=onsocial_sign_in(OauthClientName::Github)
                icon=Some(AiGithubOutlined)
                icon_before=true
            />
        </div>

        <div class="w-full max-w-md flex items-center my-6">
            <hr class="flex-grow border-t border-mid-gray"/>
            <span class="mx-4">"OR"</span>
            <hr class="flex-grow border-t border-mid-gray"/>
        </div>

        <ReactiveForm form_ref=login_form_ref on:submit=handle_submit ext_styles="w-full max-w-md">
            <div class="mb-6">
                <InputField
                    label="Email/Username"
                    field_type=InputFieldType::Text
                    name="user_name"
                    required=true
                    placeholder="Enter your email or username"
                    id_attr="user_name"
                    ext_input_styles="focus:ring-secondary"
                    autocomplete="on"
                />
            </div>
            <div class="mb-6">
                <InputField
                    label="Password"
                    field_type=InputFieldType::Password
                    name="password"
                    required=true
                    placeholder="Enter your password"
                    id_attr="password"
                    ext_input_styles="focus:ring-secondary"
                    autocomplete="on"
                />
            </div>

            <div class="flex items-center justify-between mb-6">
                <a class="text-sm text-secondary hover:secondary" href="#">"Forgot Password?"</a>
            </div>

            <BasicButton
                button_text="Sign In"
                style_ext="bg-primary text-contrast-white px-4 py-2 hover:bg-primary transition duration-300 ease-in-out hover:shadow-md hover:-translate-y-1 hover:z-10 text-contrast-white w-full"
                button_type=ButtonType::Submit
                disabled=submit_is_disabled
            />

            {on_switch_to_sign_up.map(|cb| view! {
                <button
                    type="button"
                    class="flex items-center justify-center mt-6 text-sm text-secondary hover:text-primary w-full"
                    on:click=move |_| cb.run(())
                >
                    "Don't have an account? Sign up"
                </button>
            })}
        </ReactiveForm>
    }
}
