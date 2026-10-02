use chrono::Local;
use icondata::{AiFilePdfOutlined, AiReadOutlined, BsArrowRight, MdiFileDocumentEditOutline};
use leptos::ev::{self, SubmitEvent};
use leptos::html::Form;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_router::hooks::use_location;
use web_sys::{HtmlFormElement, HtmlInputElement, HtmlSelectElement};

use detaxine_ui::{
    components::{
        actions::button::BasicButton,
        feedback::{
            modal::modal::{BasicModal, UseCase},
            spinner::Spinner,
        },
        forms::{
            checkbox::CheckboxInputField,
            datepicker::DatePicker,
            input::{CustomFileInput, InputField, InputFieldType},
            reactive_form::ReactiveForm,
            select::{SelectInput, SelectOption},
            textarea::Textarea,
        },
        navigation::stepper::{Step, StepInfo, Stepper},
    },
    utils::{
        formatters::{Pipe, PipeOption},
        forms::{FormDeserializeOptions, deserialize_form_with_options},
    },
};

use crate::components::molecules::auth::auth_modal::AuthModal;
use crate::data::{
    context::{auth::use_auth, billing::use_billing},
    models::graphql::shared::{
        BillingInterval, BillingIntervalForm, FetchBillingRateVars, ServiceIdsForm,
        ServiceRequestInput, UserService,
    },
};
use crate::utils::custom_traits::EnumerableEnum;
use crate::views::public::error_handler::ErrorHandler;

#[component]
pub fn RatecardComponent(
    #[prop(into)] name: RwSignal<String>,
    #[prop(into)] services: RwSignal<Vec<UserService>>,
) -> impl IntoView {
    let services_form_ref = NodeRef::new();
    let billing_interval_form_ref = NodeRef::new();
    let billing_interval_field_ref = NodeRef::new();
    let file_input_ref = NodeRef::new();
    let (services_form_is_valid, set_services_form_is_valid) = signal(false);
    let (billing_interval_form_is_valid, set_billing_interval_form_is_valid) = signal(false);
    let submit_is_disabled =
        Memo::new(move |_| !services_form_is_valid.get() || !billing_interval_form_is_valid.get());
    let location = use_location();
    let billing_ctx = use_billing();

    let success_modal_is_open = RwSignal::new(false);
    let service_request_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);
    let auth_modal_is_open = RwSignal::new(false);

    let stepper_form_refs = RwSignal::new(Vec::new());

    let modal_primary_is_disabled = Memo::new(move |_| {
        let refs = stepper_form_refs.get();
        if refs.is_empty() {
            return true;
        }
        refs.iter().any(|form_ref: &NodeRef<Form>| {
            form_ref
                .get()
                .map(|form| !form.check_validity())
                .unwrap_or(true)
        })
    });

    let handle_received_form_refs = Callback::new(move |form_refs: Vec<NodeRef<Form>>| {
        stepper_form_refs.update(|prev| *prev = form_refs);
    });

    let billing_interval = RwSignal::new(
        BillingInterval::variants_slice()
            .iter()
            .map(|billing_interval| {
                SelectOption::new(
                    &format!("{billing_interval:?}"),
                    &billing_interval.to_string(),
                )
            })
            .collect::<Vec<SelectOption>>(),
    );
    let (selected_billing_interval, set_selected_billing_interval) = signal("hr");

    // Each card owns its own billing rate. Prevents the shared-signal
    // bug where every mounted RatecardComponent displayed the amount
    // from whichever card most recently triggered a fetch.
    let local_billing_rate = RwSignal::new(None::<String>);

    let amount = Memo::new(move |_| local_billing_rate.get().and_then(|s| s.parse::<f64>().ok()));

    // Owned by the component, not the effect run — otherwise the callback's
    // StoredValue gets disposed when the effect re-runs, and the in-flight
    // async task panics when it calls .run() on the disposed handle.
    let on_billing_rate = Callback::new(move |result: Option<String>| {
        local_billing_rate.set(result);
    });

    // Fire the billing-rate query whenever both forms become valid.
    Effect::new(move |_| {
        if let Some(form) = billing_interval_form_ref.get() as Option<HtmlFormElement> {
            set_billing_interval_form_is_valid.set(form.check_validity());
        }

        if !(services_form_is_valid.get() && billing_interval_form_is_valid.get()) {
            return;
        }

        let Some(billing_data) = deserialize_form_with_options::<BillingIntervalForm>(
            &billing_interval_form_ref,
            &Default::default(),
        ) else {
            return;
        };
        let Some(services_data) = deserialize_form_with_options::<ServiceIdsForm>(
            &services_form_ref,
            &FormDeserializeOptions {
                vec_fields: Some(&["service_ids"]),
                ..Default::default()
            },
        ) else {
            return;
        };

        let vars = FetchBillingRateVars {
            billing_interval: billing_data.billing_interval,
            service_ids: services_data.service_ids,
        };

        billing_ctx.fetch_billing_rate(vars, on_billing_rate);
    });

    // React to successful service-request creation.
    Effect::new(move |prev: Option<u64>| {
        let dirty = billing_ctx.service_request_created_dirty.get();
        if let Some(prev) = prev {
            if dirty != prev {
                if let Some(form_ref) = stepper_form_refs.get_untracked().first() {
                    if let Some(form) = form_ref.get() {
                        form.reset();
                    }
                }
                success_modal_is_open.set(true);
                service_request_modal_is_open.set(false);
            }
        }
        dirty
    });

    let handle_services_form_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        if let Some(form) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        {
            set_services_form_is_valid.set(form.check_validity());
        }
    };

    let handle_billing_interval_form_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        if let Some(form) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        {
            set_billing_interval_form_is_valid.set(form.check_validity());
        }
    };

    let handle_service_request_modal_primary_click = Callback::new(move |_| {
        confirm_modal_is_open.update(|status| *status = true);
    });

    let onprimary_confirm_handler = Callback::new(move |_| {
        if modal_primary_is_disabled.get() {
            return;
        }

        let Some(file_input) = file_input_ref.get() as Option<HtmlInputElement> else {
            return;
        };

        let mut files = Vec::new();
        if let Some(list) = file_input.files() {
            for i in 0..list.length() {
                if let Some(file) = list.item(i) {
                    files.push(file);
                }
            }
        }

        let Some(service_request_form_ref) = stepper_form_refs.get_untracked().first().cloned()
        else {
            return;
        };

        let Some(services_data) = deserialize_form_with_options::<ServiceIdsForm>(
            &services_form_ref,
            &FormDeserializeOptions {
                vec_fields: Some(&["service_ids"]),
                ..Default::default()
            },
        ) else {
            return;
        };

        let Some(service_request_input) = deserialize_form_with_options::<ServiceRequestInput>(
            &service_request_form_ref,
            &FormDeserializeOptions {
                numeric_fields: Some(&["engagement_length"]),
                ..Default::default()
            },
        ) else {
            return;
        };

        let redirect_to = location.pathname.get();

        billing_ctx.create_service_request(
            files,
            service_request_input,
            services_data.service_ids,
            Some(redirect_to),
        );
    });

    let handle_stepper_on_cleanup = Callback::new(move |_| {
        stepper_form_refs.update(|refs| refs.clear());
    });

    view! {
        <ErrorHandler unauthorized_cb=Callback::new(move |_| auth_modal_is_open.set(true)) />
        <AuthModal is_open=auth_modal_is_open />
        <div class="flex flex-col gap-[20px] border-[0.5px] border-light-gray rounded-[5px] min-h-[564px] max-w-[400px] flex-1">
            <BasicModal title="Service Request" is_open=service_request_modal_is_open use_case=UseCase::General disable_auto_close=false class="w-[70%] md:w-[60%] h-[70svh]" show_footer=false>
                <>
                <Show when=move || billing_ctx.is_loading.get()>
                    <Spinner />
                </Show>
                <Stepper step_labels=RwSignal::new(vec![StepInfo::new("Basic Information", Some(MdiFileDocumentEditOutline)), StepInfo::new("Supporting Documents", Some(AiFilePdfOutlined)), StepInfo::new("Review", Some(AiReadOutlined))]) send_all_form_refs=handle_received_form_refs is_linear=true final_button_text="Submit" ext_wrapper_styles="h-full overflow-y-auto" on_click_final_button=handle_service_request_modal_primary_click final_button_is_disabled=modal_primary_is_disabled handle_on_cleanup=handle_stepper_on_cleanup>
                   <Step>
                       <div class="flex flex-col gap-[20px] p-2">
                           <Textarea label="Description" required=true id_attr="description" name="description" placeholder="Describe your request" />
                           <DatePicker label="Start Date" required=true id_attr="start_date" name="start_date" min=Local::now() />
                           {
                               move || {
                                   let mut interval_str = "";
                                   if let Some(input_el) = billing_interval_field_ref.get() as Option<HtmlSelectElement> {
                                       interval_str = match input_el.value().as_str() {
                                           "Monthly" => "months",
                                           "Hourly" => "hours",
                                           "Weekly" => "weeks",
                                           "Annual" => "years",
                                           "Milestone" => "milestones",
                                           _ => "_ _",
                                       };
                                   };
                                   view! {
                                       <InputField label=format!("Engagement Length ({})", interval_str) min="1" field_type=InputFieldType::Number required=true id_attr="engagement_length" name="engagement_length" placeholder="e.g. 1" />
                                   }
                               }
                           }
                       </div>
                   </Step>
                   <Step>
                       <div class="flex flex-col gap-[20px] p-2">
                           <CustomFileInput input_node_ref=file_input_ref label="Supporting Documents" name="supporting_documents" id_attr="supporting_documents" accept="image/*, .pdf, .docx, .txt, .odt, .md" required=true multiple=true />
                       </div>
                   </Step>
                   <Step>
                        <div class="flex flex-col gap-[20px]">
                            { move || if let Some(first_form_ref) = stepper_form_refs.get().get(0) {
                                let Some(data) = deserialize_form_with_options::<ServiceRequestInput>(&first_form_ref, &Default::default()) else { return None };
                                Some(view! {
                                    <h4>"Basic Information"</h4>
                                    <table class="border-collapse border border-light-gray dark:border-mid-gray">
                                        <tr>
                                            <td class="border-collapse border border-light-gray dark:border-mid-gray px-4 py-2"><strong>"Description"</strong></td>
                                            <td class="border-collapse border border-light-gray dark:border-mid-gray px-4 py-2">{data.description.text(None)}</td>
                                        </tr>
                                        <tr>
                                            <td class="border-collapse border border-light-gray dark:border-mid-gray px-4 py-2"><strong>"Start Date"</strong></td>
                                            <td class="border-collapse border border-light-gray dark:border-mid-gray px-4 py-2">{data.start_date.date("%b %e %Y", None)}</td>
                                        </tr>
                                        <tr>
                                            <td class="border-collapse border border-light-gray dark:border-mid-gray px-4 py-2"><strong>"Engagement Length"</strong></td>
                                            <td class="border-collapse border border-light-gray dark:border-mid-gray px-4 py-2">{data.engagement_length.int(None)}</td>
                                        </tr>
                                    </table>
                                })
                            } else {
                                None
                            }
                            }
                            { move ||
                                if let Some(_second_form_ref) = stepper_form_refs.get().get(1) {
                                    let Some(file_input) = file_input_ref.to_owned().get() else { return None };

                                    let files = file_input.files();
                                    let file_list = files.map(|fl| {
                                        (0..fl.length())
                                            .filter_map(|i| fl.item(i))
                                            .collect::<Vec<_>>()
                                    }).unwrap_or_default();

                                    Some(view! {
                                        <h4>"Supporting Documents"</h4>
                                        {if file_list.is_empty() {
                                            view! {
                                                <p class="text-light-gray dark:text-mid-gray italic">"No documents uploaded."</p>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <table class="border-collapse border border-light-gray dark:border-mid-gray">
                                                    <thead>
                                                        <tr>
                                                            <th class="border-collapse border border-light-gray dark:border-mid-gray px-4 py-2 text-left"><strong>"File Name"</strong></th>
                                                            <th class="border-collapse border border-light-gray dark:border-mid-gray px-4 py-2 text-left"><strong>"Size"</strong></th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {file_list.into_iter().map(|file| {
                                                            let name = file.name();
                                                            let size = file.size();

                                                            let size_display = if size < 1024.0 {
                                                                format!("{:.0} B", size)
                                                            } else if size < 1024.0 * 1024.0 {
                                                                format!("{:.1} KB", size / 1024.0)
                                                            } else {
                                                                format!("{:.1} MB", size / (1024.0 * 1024.0))
                                                            };

                                                            view! {
                                                                <tr>
                                                                    <td class="border-collapse border border-light-gray dark:border-mid-gray px-4 py-2">{name}</td>
                                                                    <td class="border-collapse border border-light-gray dark:border-mid-gray px-4 py-2">{size_display}</td>
                                                                </tr>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </tbody>
                                                </table>
                                            }.into_any()
                                        }}
                                    })
                                } else {
                                    None
                                }
                            }
                       </div>
                   </Step>
                </Stepper>
                </>
            </BasicModal>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Service Request submitted successfully!"</p>
                    <p>"Elon will reach out to you shortly."</p>
                </div>
            </BasicModal>
            <BasicModal title="Confirm" on_click_primary=onprimary_confirm_handler is_open=confirm_modal_is_open use_case=UseCase::Confirmation disable_auto_close=false stack_number=1>
                <div class="p-[10px]">
                    <p>"Are you sure that you want to submit?"</p>
                </div>
            </BasicModal>
            <div class="border-b-[0.5px]">
                <div class="p-[10px] flex flex-row justify-between items-center">
                    <div class="flex flex-col">
                        <h4>{move || name.get()}</h4>
                        <p class="text-primary font-bold text-2xl"><sup class="text-sm">$</sup>{ move || amount.get().float(Some(2), Some("_ _")) }/{move || selected_billing_interval.get()}</p>
                    </div>
                    <div class="basis-1/3">
                        <ReactiveForm on:submit=handle_billing_interval_form_submit form_ref=billing_interval_form_ref>
                            <SelectInput
                                id_attr="billing_interval"
                                name="billing_interval"
                                options=billing_interval
                                required=true
                                initial_value="Hourly".to_string()
                                ext_input_styles=""
                                input_node_ref=billing_interval_field_ref
                                on:change=move |ev: ev::Event| {
                                    let target = ev.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
                                    if let Some(input_el) = target {
                                        let short_name = match input_el.value().as_str() {
                                            "Monthly" => "mo",
                                            "Hourly" => "hr",
                                            "Weekly" => "wk",
                                            "Annual" => "yr",
                                            "Milestone" => "mi",
                                            _ => "_ _",
                                        };
                                        set_selected_billing_interval.set(short_name);
                                    }
                                }
                            />
                        </ReactiveForm>
                    </div>
                </div>
            </div>

            <ReactiveForm
                on:submit=handle_services_form_submit
                form_ref=services_form_ref
                on:change=move |_| {
                    if let Some(form) = services_form_ref.get() {
                        let has_checked = form
                            .query_selector_all("input[name='service_ids']:checked")
                            .map(|nodes| nodes.length() > 0)
                            .unwrap_or(false);
                        set_services_form_is_valid.set(has_checked);
                    }
                }
            >
                <div class="p-[10px] flex flex-col gap-[10px] text-md">
                    <For
                        each=move || services.get()
                        key=|service| service.id.as_ref().unwrap_or(&String::new()).clone()
                        children=move |service| {
                            view! {
                                <CheckboxInputField initial_value=RwSignal::new(service.id.as_ref().unwrap_or(&String::new()).clone()) label=service.title.as_ref().unwrap_or(&String::new()).clone() id_attr=format!("service-{}", service.id.as_ref().unwrap_or(&String::new()).clone()) name="service_ids" />
                            }
                        }
                    />
                </div>
            </ReactiveForm>
            <div class="p-[10px] mt-auto">
                <BasicButton button_text="Request Service" icon=Some(BsArrowRight) style_ext="bg-primary text-contrast-white" disabled=submit_is_disabled onclick=Callback::new(move |_| { service_request_modal_is_open.set(true); }) />
            </div>
        </div>
    }.into_any()
}
