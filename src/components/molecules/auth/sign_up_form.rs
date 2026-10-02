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
    context::{auth::use_auth, ui::use_ui, user::use_user},
    models::{
        general::acl::{AuthCode, OauthClientName},
        graphql::acl::UserInput,
    },
};

/// Sign-up form. Fires `on_success` when `user_created_dirty` bumps.
/// `on_switch_to_sign_in` is optional — pages that link to `/sign-in` via a
/// route can ignore it; the modal passes it to swap tabs in place.
#[component]
pub fn SignUpForm(
    #[prop(optional)] on_success: Option<Callback<()>>,
    #[prop(optional)] on_switch_to_sign_in: Option<Callback<()>>,
) -> impl IntoView {
    let signup_form_ref = NodeRef::new();
    let auth_ctx = use_auth();
    let user_ctx = use_user();
    let ui_ctx = use_ui();

    let (form_is_valid, set_form_is_valid) = signal(false);
    let (confirm_password_value, set_confirm_password_value) = signal(None::<String>);
    let (password_is_matching, set_password_is_matching) = signal(false);
    let submit_is_disabled =
        Memo::new(move |_| !form_is_valid.get() || !password_is_matching.get());

    // OAuth callback + redirect effects — same as SignInForm.
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

    Effect::new(move |_| {
        if let Some(url) = auth_ctx.oauth_redirect_url.get() {
            if let Some(window) = window() {
                let _ = window.open_with_url_and_target(&url, "_self");
            }
        }
    });

    // Fire on_success when user_created_dirty transitions.
    Effect::new(move |prev: Option<u64>| {
        let dirty = user_ctx.user_created_dirty.get();

        if let Some(prev) = prev {
            if dirty != prev {
                if let Some(form) = signup_form_ref
                    .get_untracked()
                    .and_then(|el: HtmlFormElement| el.dyn_into::<HtmlFormElement>().ok())
                {
                    form.reset();
                    set_form_is_valid.set(false);
                }
                set_confirm_password_value.set(None);
                ui_ctx.error.set(None);
                if let Some(cb) = on_success {
                    cb.run(());
                }
            }
        }
        dirty
    });

    let onsocial_sign_up = move |client: OauthClientName| {
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

        let Some(user_input) = deserialize_form_with_options::<UserInput>(
            &signup_form_ref,
            &FormDeserializeOptions {
                deserialize_bool: true,
                ..Default::default()
            },
        ) else {
            return;
        };

        user_ctx.create_user(user_input);
    };

    // Password-match validation.
    Effect::new(move || {
        let Some(confirmed_password) = confirm_password_value.get() else {
            return;
        };

        let Some(deserialized) = deserialize_form_with_options::<UserInput>(
            &signup_form_ref,
            &FormDeserializeOptions {
                deserialize_bool: true,
                ..Default::default()
            },
        ) else {
            return;
        };

        set_password_is_matching.set(confirmed_password == deserialized.password);
    });

    view! {
        <div class="w-full max-w-md flex flex-col items-center gap-2 my-4">
            <BasicButton
                button_text="Continue with Google"
                style_ext="border-[1px] border-danger hover:bg-danger transition-all duration-300 ease-in-out hover:shadow-md hover:-translate-y-1 hover:z-10 hover:text-contrast-white text-danger w-full"
                onclick=onsocial_sign_up(OauthClientName::Google)
                icon=Some(AiGoogleOutlined)
                icon_before=true
            />
            <BasicButton
                button_text="Continue with GitHub"
                style_ext="border-[1px] border-gray hover:bg-gray transition-all duration-300 ease-in-out hover:shadow-md hover:-translate-y-1 hover:z-10 hover:text-contrast-white w-full"
                onclick=onsocial_sign_up(OauthClientName::Github)
                icon=Some(AiGithubOutlined)
                icon_before=true
            />
        </div>

        <div class="w-full max-w-md flex items-center my-6">
            <hr class="flex-grow border-t border-mid-gray"/>
            <span class="mx-4">"OR"</span>
            <hr class="flex-grow border-t border-mid-gray"/>
        </div>

        <ReactiveForm form_ref=signup_form_ref on:submit=handle_submit ext_styles="w-full max-w-md">
            <div class="mb-6">
                <InputField
                    label="Email"
                    field_type=InputFieldType::Email
                    name="email"
                    required=true
                    placeholder="Enter your email"
                    id_attr="email"
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
                    placeholder="Choose a password"
                    id_attr="password"
                    ext_input_styles="focus:ring-secondary"
                />
            </div>
            <div class="mb-6">
                <InputField
                    label="Confirm Password"
                    field_type=InputFieldType::Password
                    required=true
                    placeholder="Repeat your password"
                    id_attr="confirm_password"
                    ext_input_styles="focus:ring-secondary"
                    on:input=move |e| set_confirm_password_value.set(Some(event_target_value(&e)))
                />
                <p class=move || format!("text-xs py-[5px] h-[30px] {}", if password_is_matching.get() { "text-success" } else { "text-danger" })>
                    {move || confirm_password_value.get().is_some().then(|| {
                        format!(
                            "{}",
                            if password_is_matching.get() {
                                "Your passwords are matching!"
                            } else {
                                "Your passwords are not matching!"
                            }
                        )
                    })}
                </p>
            </div>
            <BasicButton
                button_text="Create Account"
                style_ext="bg-primary text-contrast-white px-4 py-2 hover:bg-primary transition duration-300 ease-in-out hover:shadow-md hover:-translate-y-1 hover:z-10 text-contrast-white w-full"
                button_type=ButtonType::Submit
                disabled=submit_is_disabled
            />

            {on_switch_to_sign_in.map(|cb| view! {
                <button
                    type="button"
                    class="flex items-center justify-center mt-6 text-sm text-secondary hover:text-primary w-full"
                    on:click=move |_| cb.run(())
                >
                    "Already have an account? Sign in"
                </button>
            })}
        </ReactiveForm>
    }
}
