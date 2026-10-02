use std::collections::HashMap;

use detaxine_ui::{
    components::{
        actions::button::{BasicButton, ButtonType},
        data_display::{
            table::data_table::{Column, DataTable, TableCellData},
            tag::LabelTag,
        },
        feedback::{
            modal::modal::{BasicModal, UseCase},
            spinner::Spinner,
        },
        forms::{
            datepicker::DatePicker,
            input::{InputField, InputFieldType},
            radio_input::{RadioInputGroup, RadioOption},
            reactive_form::ReactiveForm,
            textarea::Textarea,
        },
        navigation::breadcrumbs::Breadcrumbs,
        schemas::props::ColorTemperature,
    },
    utils::forms::{FormDeserializeOptions, deserialize_form_with_options},
};
use icondata::BsPlusLg;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_meta::*;
use leptos_router::components::{A, Outlet};
use web_sys::HtmlFormElement;

use crate::data::{
    context::portfolio::use_portfolio, models::graphql::shared::UserProfessionalInfoInput,
};

const SHARED_SERVICE_API: Option<&str> = option_env!("SHARED_SERVICE_API");

#[component]
pub fn ProfessionalDetails() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn ProfessionalDetailsList() -> impl IntoView {
    let portfolio_ctx = use_portfolio();
    let professions = move || portfolio_ctx.professions;

    let table_data = RwSignal::new((
        vec![
            Column::new("Occupation", false),
            Column::new("Status", true),
            Column::new("Start Date", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        portfolio_ctx.fetch_professions();
    });

    Effect::new(move || {
        let roles: Vec<HashMap<String, TableCellData>> = professions()
            .get()
            .iter()
            .map(|profession| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".to_string(),
                    TableCellData::String(
                        profession
                            .id
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Occupation".to_string(),
                    TableCellData::String(
                        profession
                            .occupation
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );

                let status = if profession.active.is_some() && profession.active.unwrap_or_default()
                {
                    ViewFn::from(move || {
                        view! { <LabelTag label="Active" color=ColorTemperature::Success /> }
                    })
                } else {
                    ViewFn::from(move || {
                        view! { <LabelTag label="Inactive" color=ColorTemperature::Warning /> }
                    })
                };
                hash_map_data.insert("Status".to_string(), TableCellData::Html(status));

                hash_map_data.insert(
                    "Start Date".to_string(),
                    TableCellData::DateTime(
                        profession
                            .start_date
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data
            })
            .collect();

        table_data.update(move |prev| {
            prev.1 = roles;
        });
    });

    view! {
        <>
            <Title text="My Portfolio"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Professions"] />
            </div>
            <Show when=move || portfolio_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Professional Details</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/professional-details/create">
                    <BasicButton
                        button_text="Create"
                        icon=Some(BsPlusLg)
                        icon_before=true
                        style_ext="bg-primary text-contrast-white"
                    />
                </A>
            </div>

            <div class="display-constraints">
                <DataTable data=table_data editable=true deletable=true />
            </div>
        </>
    }
    .into_any()
}

#[component]
pub fn CreateProfessionalDetail() -> impl IntoView {
    let form_ref = NodeRef::new();
    let (form_is_valid, set_form_is_valid) = signal(false);
    let submit_is_disabled = Memo::new(move |_| !form_is_valid.get());
    let portfolio_ctx = use_portfolio();
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);

    // React to successful creation — same shape as the ACL create views.
    Effect::new(move |prev: Option<u64>| {
        let dirty = portfolio_ctx.professional_details_created_dirty.get();
        if let Some(prev) = prev {
            if dirty != prev {
                if let Some(form) = form_ref
                    .get_untracked()
                    .and_then(|el: HtmlFormElement| el.dyn_into::<HtmlFormElement>().ok())
                {
                    form.reset();
                    set_form_is_valid.set(false);
                }
                success_modal_is_open.set(true);
            }
        }
        dirty
    });

    let onprimary_handler = Callback::new(move |_| {
        if !form_is_valid.get() {
            return;
        }

        let Some(input) = deserialize_form_with_options::<UserProfessionalInfoInput>(
            &form_ref,
            &FormDeserializeOptions {
                deserialize_bool: true,
                ..Default::default()
            },
        ) else {
            return;
        };

        portfolio_ctx.create_professional_details(input);
    });

    let handle_step_form_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        if let Some(form) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        {
            set_form_is_valid.set(form.check_validity());
            if ev.submitter().is_some() {
                confirm_modal_is_open.update(|status| *status = true);
            }
        }
    };

    view! {
        <>
            <Title text="New Profession"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Profession created successfully!"</p>
                </div>
            </BasicModal>
            <BasicModal title="Confirm" on_click_primary=onprimary_handler is_open=confirm_modal_is_open use_case=UseCase::Confirmation disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Are you sure that you want to submit?"</p>
                </div>
            </BasicModal>
            <Show when=move || portfolio_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Professions", "New"] />
            </div>

            <h1 class="display-constraints">New Profession</h1>

            <ReactiveForm on:submit=handle_step_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Text label="Occupation" required=true id_attr="occupation" name="occupation" />
                    <Textarea label="Description" required=true id_attr="description" name="description" />

                    <DatePicker label="Start Date" required=true id_attr="start_date" name="start_date" />
                    <RadioInputGroup
                        legend="Select Status"
                        name="active"
                        required=true
                        options=vec![
                            RadioOption::new("true", "Active", None),
                            RadioOption::new("false", "InActive", None),
                        ]
                    />
                    <BasicButton
                        button_text="Submit"
                        style_ext="bg-primary text-contrast-white"
                        button_type=ButtonType::Submit
                        disabled=submit_is_disabled
                    />
                </div>
            </ReactiveForm>
        </>
    }.into_any()
}
