use detaxine_ui::{
    components::{
        actions::button::{BasicButton, ButtonType},
        feedback::modal::modal::{BasicModal, UseCase},
        forms::{
            input::{InputField, InputFieldType},
            reactive_form::ReactiveForm,
        },
    },
    utils::forms::deserialize_form_with_options,
};
use icondata::BsEnvelope;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::{HtmlFormElement, SubmitEvent};

use crate::data::context::{shared::use_shared, ui::UiContext};
use crate::data::models::graphql::email::SubscriberInput;

#[component]
pub fn WaitList() -> impl IntoView {
    let ui = expect_context::<UiContext>();
    let shared = use_shared();

    let subscription_form_ref = NodeRef::new();
    let (form_is_valid, set_form_is_valid) = signal(false);
    let subscribe_button_is_disabled =
        Memo::new(move |_| !form_is_valid.get() || shared.is_loading.get());
    let success_modal_is_open = RwSignal::new(false);

    let handle_subscribe_form_submit = move |ev: SubmitEvent| {
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

        let Some(subscriber) = deserialize_form_with_options::<SubscriberInput>(
            &subscription_form_ref,
            &Default::default(),
        ) else {
            ui.set_client_error("Could not read the form. Please try again.");
            return;
        };

        let form_ref = subscription_form_ref;
        shared.subscribe_to_marketplace_waitlist(
            subscriber,
            Callback::new(move |success: bool| {
                if !success {
                    return;
                }

                if let Some(form) = form_ref
                    .get_untracked()
                    .and_then(|el| el.dyn_into::<HtmlFormElement>().ok())
                {
                    form.reset();
                    set_form_is_valid.set(false);
                }
                success_modal_is_open.set(true);
            }),
        );
    };

    view! {
        <div class="flex flex-col gap-[40px] min-h-svh">
            <div>
                <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                    <div class="p-[10px]">
                        <p>"You have successfully subscribed to our marketplace launch update!"</p>
                        <p>"We guarantee that you will only receive updates when the marketplace is launched."</p>
                    </div>
                </BasicModal>

                <div class="display-constraints">

                    // Hero
                    <div class="max-w-2xl mb-20">
                        <span class="inline-block text-xs tracking-[0.3em] uppercase text-primary border border-primary px-3 py-1 rounded-[5px] mb-6">
                            "Coming Soon"
                        </span>
                        <h1 class="mb-6">
                            "The Marketplace for"
                            <span class="text-primary"> " IoT builders,"</span>
                            " Leptos devs & ambitious learners."
                        </h1>
                        // Hero subtext
                        <p class="text-body mb-10 max-w-xl">
                            "Edge-optimized backend services built around MQTT. Point them at your MQTT broker and SurrealDB instance and they just work - no vendor lock-in, no cloud dependency. We distribute them as small multi-arch binaries and/or hardened Docker images. Our shop will also have Leptos templates, Ebooks, and courses for Rust developers who aspire to ship real products."
                        </p>

                        // Email form
                        <ReactiveForm form_ref=subscription_form_ref on:submit=handle_subscribe_form_submit>
                            <div class="flex flex-col sm:flex-row gap-3 max-w-md">
                                <InputField field_type=InputFieldType::Email placeholder="Enter your email" icon=BsEnvelope required=true id_attr="email" name="email" />
                                <BasicButton
                                    button_text="Join Waitlist"
                                    style_ext="bg-primary text-contrast-white hover:bg-secondary"
                                    button_type=ButtonType::Submit
                                    disabled=subscribe_button_is_disabled
                                />
                            </div>
                        </ReactiveForm>

                        <p class="text-xs mt-3">"No spam. Unsubscribe anytime."</p>
                    </div>

                    // Divider
                    <div class="flex flex-col gap-[5px] mb-20">
                        <div class="h-px bg-light-gray dark:bg-mid-gray"></div>
                    </div>

                    // What's in the marketplace
                    <div class="mb-20">
                        <h6 class="mb-10">"What's Coming?"</h6>
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                        {[
                            ("ACL", "Fine-grained access control as a service. Roles, permissions, and resource guards - ready to plug into any stack. Much like AWS IAM. Comes with a free GUI and request tracer.", "https://techietenka.com/api/files/view/default/medium-vecteezy_a-man-and-virtual-icon-modern-cyberspace-network-technology_24162388_medium.jpg?width=600"),
                            ("Storage", "Edge-optimized file storage with upload, retrieval, transformation and CDN delivery out of the box. Much like AWS S3. Comes with a free GUI and request tracer.", "https://techietenka.com/api/files/view/default/medium-vecteezy_a-man-and-virtual-icon-modern-cyberspace-network-technology_24162388_medium.jpg?width=600"),
                            ("Email", "Transactional and bulk email service with mailing lists, templates, campaigns, and delivery analytics. Much like AWS SES. Comes with a free GUI and request tracer.", "https://techietenka.com/api/files/view/default/medium-vecteezy_a-man-and-virtual-icon-modern-cyberspace-network-technology_24162388_medium.jpg?width=600"),
                            ("Leptos Templates", "Production-grade full-stack Leptos starter kits and UI component libraries.", "https://techietenka.com/api/files/view/default/medium-vecteezy_a-man-and-virtual-icon-modern-cyberspace-network-technology_24162388_medium.jpg?width=600"),
                            ("Ebooks", "Deep-dive guides on Rust, Leptos, IoT, embedded systems, and systems design.", "https://techietenka.com/api/files/view/default/medium-vecteezy_a-man-and-virtual-icon-modern-cyberspace-network-technology_24162388_medium.jpg?width=600"),
                            ("Courses", "Structured video and text courses that take you from zero to production on Rust, Leptos and IoT.", "https://techietenka.com/api/files/view/default/medium-vecteezy_a-man-and-virtual-icon-modern-cyberspace-network-technology_24162388_medium.jpg?width=600"),
                        ].into_iter().map(|(title, desc, img)| view! {
                            <div class="relative overflow-hidden h-[200px] rounded-[5px] border-[1px] border-mid-gray">
                                <div
                                    class="absolute inset-0 bg-cover bg-center"
                                    style=format!("background-image: url('{}');", img)
                                />
                                <div class="absolute inset-0 bg-navy/60" />
                                <div
                                    class="absolute inset-0 bg-primary"
                                    style="clip-path: polygon(0 0, 65% 0, 45% 100%, 0 100%);"
                                />
                                <div class="relative z-10 flex flex-col h-full p-6 h-[200px]">
                                    <h5 class="text-contrast-white mb-2">{title}</h5>
                                    <p class="text-caption text-contrast-white/70">{desc}</p>
                                </div>
                            </div>
                        }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }.into_any()
}
