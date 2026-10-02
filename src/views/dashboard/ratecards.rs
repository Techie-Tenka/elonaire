use std::collections::HashMap;

use detaxine_ui::{
    components::{
        actions::button::{BasicButton, ButtonType},
        data_display::table::data_table::{Column, DataTable, TableCellData},
        feedback::{
            modal::modal::{BasicModal, UseCase},
            spinner::Spinner,
        },
        forms::{
            input::{InputField, InputFieldType},
            reactive_form::ReactiveForm,
            select::{CustomSelectInput, SelectOption},
        },
        navigation::breadcrumbs::Breadcrumbs,
    },
    utils::forms::deserialize_form_with_options,
};
use icondata::BsPlusLg;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_meta::*;
use leptos_router::components::{A, Outlet};
use web_sys::HtmlFormElement;

use crate::data::{
    context::{auth::use_auth, billing::use_billing, portfolio::use_portfolio},
    models::graphql::shared::RatecardInput,
};

#[component]
pub fn Ratecards() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn RatecardsList() -> impl IntoView {
    let billing_ctx = use_billing();
    let ratecards = move || billing_ctx.ratecards;

    let table_data = RwSignal::new((
        vec![
            Column::new("Name", false),
            Column::new("Date of Creation", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        billing_ctx.fetch_ratecards();
    });

    Effect::new(move || {
        let ratecards_rows: Vec<HashMap<String, TableCellData>> = ratecards()
            .get()
            .iter()
            .map(|ratecard| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".into(),
                    TableCellData::String(
                        ratecard
                            .id
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Name".into(),
                    TableCellData::String(
                        ratecard
                            .name
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Date of Creation".into(),
                    TableCellData::DateTime(
                        ratecard
                            .created_at
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data
            })
            .collect();

        table_data.update(move |prev| {
            prev.1 = ratecards_rows;
        });
    });

    view! {
        <>
            <Title text="Rate Cards"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Rate Cards"] />
            </div>
            <Show when=move || billing_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Rate Cards</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/ratecards/create">
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
pub fn CreateRatecard() -> impl IntoView {
    let form_ref = NodeRef::new();
    let (main_form_is_valid, set_main_form_is_valid) = signal(false);
    let selected_services_options = RwSignal::new(vec![] as Vec<String>);
    let submit_is_disabled =
        Memo::new(move |_| !main_form_is_valid.get() || selected_services_options.get().is_empty());
    let billing_ctx = use_billing();
    let portfolio_ctx = use_portfolio();
    let auth_ctx = use_auth();
    let services = move || portfolio_ctx.services;
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);
    let services_options = RwSignal::new(vec![] as Vec<SelectOption>);

    // React to successful creation.
    Effect::new(move |prev: Option<u64>| {
        let dirty = billing_ctx.ratecard_created_dirty.get();
        if let Some(prev) = prev {
            if dirty != prev {
                if let Some(form) = form_ref
                    .get_untracked()
                    .and_then(|el: HtmlFormElement| el.dyn_into::<HtmlFormElement>().ok())
                {
                    form.reset();
                    set_main_form_is_valid.set(false);
                }
                selected_services_options.set(vec![]);
                success_modal_is_open.set(true);
            }
        }
        dirty
    });

    let onprimary_handler = Callback::new(move |_| {
        if selected_services_options.get().is_empty() || !main_form_is_valid.get() {
            return;
        }

        let Some(ratecard_input) =
            deserialize_form_with_options::<RatecardInput>(&form_ref, &Default::default())
        else {
            return;
        };

        billing_ctx.create_ratecard(ratecard_input, selected_services_options.get_untracked());
    });

    // Fetch the services dropdown source on mount.
    Effect::new(move |_| {
        portfolio_ctx.fetch_services();
    });

    // Derive the select options from the portfolio signal.
    Effect::new(move |_| {
        services_options.set(
            services()
                .get()
                .iter()
                .map(|service| {
                    SelectOption::new(
                        service.id.as_ref().unwrap_or(&Default::default()),
                        service.title.as_ref().unwrap_or(&Default::default()),
                    )
                })
                .collect(),
        );
    });

    let handle_main_form_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        ev.stop_propagation();

        if let Some(form) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        {
            set_main_form_is_valid.set(form.check_validity());

            if ev.submitter().is_some() {
                confirm_modal_is_open.update(|status| *status = true);
            }
        }
    };

    view! {
        <>
            <Title text="New Rate Card"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Rate Card created successfully!"</p>
                </div>
            </BasicModal>
            <BasicModal title="Confirm" on_click_primary=onprimary_handler is_open=confirm_modal_is_open use_case=UseCase::Confirmation disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Are you sure that you want to submit?"</p>
                </div>
            </BasicModal>
            <Show when=move || billing_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Rate Cards", "New"] />
            </div>

            <h1 class="display-constraints">New Rate Card</h1>

            <div class="display-constraints flex flex-col gap-[20px]">
                <h2>Rate Card Metadata</h2>
                <CustomSelectInput
                    label="Services"
                    required=true
                    id_attr="service_ids"
                    options=services_options
                    value=selected_services_options
                    multiple=true
                />
            </div>

            <ReactiveForm on:submit=handle_main_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <h2>Rate Card Info</h2>
                    <InputField field_type=InputFieldType::Text label="Name" required=true id_attr="name" name="name" />

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
