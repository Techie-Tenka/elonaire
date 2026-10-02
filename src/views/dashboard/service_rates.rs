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
    context::{billing::use_billing, portfolio::use_portfolio},
    models::graphql::shared::ServiceRateInput,
};

#[component]
pub fn ServiceRates() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn ServiceRatesList() -> impl IntoView {
    let billing_ctx = use_billing();
    let service_rates = move || billing_ctx.service_rates;

    let table_data = RwSignal::new((
        vec![
            Column::new("Service Title", false),
            Column::new("Base Rate", true),
            Column::new("Currency", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        billing_ctx.fetch_service_rates();
    });

    Effect::new(move || {
        let service_rates_rows: Vec<HashMap<String, TableCellData>> = service_rates()
            .get()
            .iter()
            .map(|service_rate| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".into(),
                    TableCellData::String(
                        service_rate
                            .id
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Service Title".into(),
                    TableCellData::String(
                        service_rate
                            .service
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .title
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Base Rate".into(),
                    TableCellData::String(format!(
                        "{:.2}",
                        service_rate
                            .base_rate
                            .as_ref()
                            .unwrap_or(&Default::default())
                    )),
                );
                hash_map_data.insert("Currency".into(), TableCellData::String("N/A".into()));
                hash_map_data
            })
            .collect();

        table_data.update(move |prev| {
            prev.1 = service_rates_rows;
        });
    });

    view! {
        <>
            <Title text="Service Rates"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Service Rates"] />
            </div>
            <Show when=move || billing_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Service Rates</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/service-rates/create">
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
pub fn CreateServiceRate() -> impl IntoView {
    let form_ref = NodeRef::new();
    let (main_form_is_valid, set_main_form_is_valid) = signal(false);
    let selected_services_options = RwSignal::new(vec![] as Vec<String>);
    let selected_currency_options = RwSignal::new(vec![] as Vec<String>);
    let submit_is_disabled = Memo::new(move |_| {
        !main_form_is_valid.get()
            || selected_services_options.get().is_empty()
            || selected_currency_options.get().is_empty()
    });
    let billing_ctx = use_billing();
    let portfolio_ctx = use_portfolio();
    let services = move || portfolio_ctx.services;
    let currencies = move || billing_ctx.currencies;
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);
    let services_options = RwSignal::new(vec![] as Vec<SelectOption>);
    let currency_options = RwSignal::new(vec![] as Vec<SelectOption>);

    // React to successful creation.
    Effect::new(move |prev: Option<u64>| {
        let dirty = billing_ctx.service_rate_created_dirty.get();
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
                selected_currency_options.set(vec![]);
                success_modal_is_open.set(true);
            }
        }
        dirty
    });

    let onprimary_handler = Callback::new(move |_| {
        if selected_services_options.get().is_empty()
            || selected_currency_options.get().is_empty()
            || !main_form_is_valid.get()
        {
            return;
        }

        let Some(service_rate_input) =
            deserialize_form_with_options::<ServiceRateInput>(&form_ref, &Default::default())
        else {
            return;
        };

        billing_ctx.create_service_rate(
            service_rate_input,
            selected_services_options.get_untracked().join(","),
            selected_currency_options.get_untracked().join(","),
        );
    });

    // Fetch dropdown sources on mount.
    Effect::new(move |_| {
        portfolio_ctx.fetch_services();
        billing_ctx.fetch_currencies();
    });

    // Derive select options.
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

        currency_options.set(
            currencies()
                .get()
                .iter()
                .map(|currency| {
                    SelectOption::new(
                        currency.id.as_ref().unwrap_or(&Default::default()),
                        currency.name.as_ref().unwrap_or(&Default::default()),
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
            <Title text="New Service Rate"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Service Rate created successfully!"</p>
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
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "ServiceRates", "New"] />
            </div>

            <h1 class="display-constraints">New Service Rate</h1>

            <div class="display-constraints flex flex-col gap-[20px]">
                <h2>Service Rate Metadata</h2>
                <CustomSelectInput
                    label="Service"
                    required=true
                    id_attr="service_id"
                    options=services_options
                    value=selected_services_options
                />
                <CustomSelectInput
                    label="Currency"
                    required=true
                    id_attr="currency_id"
                    options=currency_options
                    value=selected_currency_options
                />
            </div>

            <ReactiveForm on:submit=handle_main_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <h2>Service Rate Info</h2>
                    <InputField field_type=InputFieldType::Number label="Base Rate" required=true id_attr="base_rate" name="base_rate" />
                    <InputField field_type=InputFieldType::Number label="Hour Week" id_attr="hour_week" name="hour_week" />

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
