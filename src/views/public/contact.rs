use detaxine_ui::{
    components::{
        actions::button::{BasicButton, ButtonType},
        feedback::modal::modal::{BasicModal, UseCase},
        forms::{
            input::{InputField, InputFieldType},
            reactive_form::ReactiveForm,
            select::{SelectInput, SelectOption},
            textarea::Textarea,
        },
    },
    utils::forms::deserialize_form_with_options,
};
use icondata::{BsEnvelope, BsGithub, BsLinkedin, BsPerson, BsSend, BsTwitterX};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;
use leptos_router::components::A;
use web_sys::{HtmlFormElement, SubmitEvent};

use crate::data::context::{shared::use_shared, ui::UiContext};
use crate::data::models::graphql::shared::{MessageInput, Subject};
use crate::utils::custom_traits::EnumerableEnum;

#[component]
pub fn Contact() -> impl IntoView {
    let ui = expect_context::<UiContext>();
    let shared = use_shared();

    let contact_form_ref = NodeRef::new();
    let (form_is_valid, set_form_is_valid) = signal(false);
    let success_modal_is_open = RwSignal::new(false);
    let submit_disabled = Memo::new(move |_| !form_is_valid.get() || shared.is_loading.get());

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        ev.stop_propagation();

        let Some(form) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        else {
            return;
        };

        let valid = form.check_validity();
        set_form_is_valid.set(valid);

        if ev.submitter().is_none() || !valid {
            return;
        }

        let Some(message) =
            deserialize_form_with_options::<MessageInput>(&contact_form_ref, &Default::default())
        else {
            ui.set_client_error("Could not read the contact form. Please try again.");
            return;
        };

        let form_ref = contact_form_ref;
        shared.send_message(
            message,
            Callback::new(move |success: bool| {
                if !success {
                    return;
                }

                if let Some(form_el) = form_ref
                    .get_untracked()
                    .and_then(|el| el.dyn_into::<HtmlFormElement>().ok())
                {
                    form_el.reset();
                    set_form_is_valid.set(false);
                }
                success_modal_is_open.set(true);
            }),
        );
    };

    let subject_options = RwSignal::new(
        Subject::variants_slice()
            .iter()
            .map(|subject| SelectOption::new(&format!("{subject:?}"), &subject.to_string()))
            .collect::<Vec<SelectOption>>(),
    );

    view! {
        <div class="min-h-svh flex flex-col gap-[40px]">
            <BasicModal
                title="Message Sent"
                is_open=success_modal_is_open
                use_case=UseCase::Success
                disable_auto_close=false
            >
                <div class="p-[10px]">
                    <p>"Thanks for reaching out! I'll get back to you as soon as possible."</p>
                </div>
            </BasicModal>

            // Header
            <div class="display-constraints blog-display-constraints border-b border-light-gray pb-8">
                <span class="inline-block text-xs tracking-[0.3em] uppercase text-primary mb-3">
                    "Contact"
                </span>
                <h1>"Get In Touch"</h1>
                <p class="text-body mt-2">
                    "Have a question, idea, or just want to say hello? I read every message."
                </p>
            </div>

            <div class="display-constraints blog-display-constraints pb-20">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-12">

                    // Left — contact details
                    <div class="md:col-span-1 flex flex-col gap-6">
                        <div>
                            <h6 class="text-primary uppercase tracking-widest mb-2">"Email"</h6>
                            <A
                                href="mailto:info@techietenka.com"
                                attr:class="text-body hover:text-primary transition-colors"
                            >
                                "info@techietenka.com"
                            </A>
                        </div>
                        <div>
                            <h6 class="text-primary uppercase tracking-widest mb-2">"Response Time"</h6>
                            <p class="text-body">"Within 48 hours"</p>
                        </div>
                        <div>
                            <h6 class="text-primary uppercase tracking-widest mb-3">"Socials"</h6>
                            <div class="flex gap-4">
                                <A href="#" attr:target="_blank" attr:class="hover:text-primary transition-colors">
                                    <Icon width="1.5rem" height="1.5rem" icon=BsLinkedin />
                                </A>
                                <A href="#" attr:target="_blank" attr:class="hover:text-primary transition-colors">
                                    <Icon width="1.5rem" height="1.5rem" icon=BsTwitterX />
                                </A>
                                <A href="#" attr:target="_blank" attr:class="hover:text-primary transition-colors">
                                    <Icon width="1.5rem" height="1.5rem" icon=BsGithub />
                                </A>
                            </div>
                        </div>
                    </div>

                    // Right — form
                    <div class="md:col-span-2">
                        <ReactiveForm form_ref=contact_form_ref on:submit=handle_submit>
                            <div class="flex flex-col gap-4">
                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                    <InputField
                                        field_type=InputFieldType::Text
                                        label="Your Name"
                                        placeholder="John Doe"
                                        name="sender_name"
                                        id_attr="sender_name"
                                        required=true
                                        icon=BsPerson
                                    />
                                    <InputField
                                        field_type=InputFieldType::Email
                                        label="Your Email"
                                        placeholder="john@example.com"
                                        name="sender_email"
                                        id_attr="sender_email"
                                        required=true
                                        icon=BsEnvelope
                                    />
                                </div>
                                <SelectInput
                                    label="Subject"
                                    name="subject"
                                    id_attr="subject"
                                    required=true
                                    options=subject_options
                                />
                                <Textarea
                                    label="Message"
                                    placeholder="Write your message here..."
                                    name="body"
                                    id_attr="body"
                                    required=true
                                    // rows=6
                                />
                                <div class="flex justify-end">
                                    <BasicButton
                                        button_text="Send Message"
                                        style_ext="bg-primary text-contrast-white hover:bg-secondary"
                                        button_type=ButtonType::Submit
                                        disabled=submit_disabled
                                        icon=Some(BsSend)
                                    />
                                </div>
                            </div>
                        </ReactiveForm>
                    </div>
                </div>
            </div>
        </div>
    }.into_any()
}
