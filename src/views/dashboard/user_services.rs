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
            input::{CustomFileInput, InputField, InputFieldType},
            reactive_form::ReactiveForm,
            textarea::Textarea,
        },
        navigation::breadcrumbs::Breadcrumbs,
    },
    utils::forms::get_form_data_from_form_ref,
};
use icondata::BsPlusLg;
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_meta::*;
use leptos_router::components::{A, Outlet};
use web_sys::{HtmlFormElement, HtmlInputElement};

use crate::data::context::portfolio::use_portfolio;

#[component]
pub fn UserService() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn UserServicesList() -> impl IntoView {
    let portfolio_ctx = use_portfolio();
    let services = move || portfolio_ctx.services;

    let table_data = RwSignal::new((
        vec![
            Column::new("Title", false),
            Column::new("Description", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        portfolio_ctx.fetch_services();
    });

    Effect::new(move || {
        let services: Vec<HashMap<String, TableCellData>> = services()
            .get()
            .iter()
            .map(|service| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".to_string(),
                    TableCellData::String(
                        service
                            .id
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Title".to_string(),
                    TableCellData::String(
                        service
                            .title
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Description".to_string(),
                    TableCellData::String(
                        service
                            .description
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );

                hash_map_data
            })
            .collect();

        table_data.update(move |prev| {
            prev.1 = services;
        });
    });

    view! {
        <>
            <Title text="User Services"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Services"] />
            </div>
            <Show when=move || portfolio_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">User Services</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/services/create">
                    <BasicButton
                        button_text="Create Service"
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
pub fn CreateUserService() -> impl IntoView {
    let form_ref = NodeRef::new();
    let file_input_ref = NodeRef::new();
    let (form_is_valid, set_form_is_valid) = signal(false);
    let submit_is_disabled = Memo::new(move |_| !form_is_valid.get());
    let portfolio_ctx = use_portfolio();
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);

    // React to successful creation.
    Effect::new(move |prev: Option<u64>| {
        let dirty = portfolio_ctx.user_service_created_dirty.get();
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

        let Some(form_data) = get_form_data_from_form_ref(&form_ref) else {
            return;
        };

        portfolio_ctx.create_user_service(files, form_data);
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
            <Title text="New Service"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Service created successfully!"</p>
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
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Services", "New Service"] />
            </div>

            <h1 class="display-constraints">New Service</h1>

            <ReactiveForm on:submit=handle_step_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Text label="Title" required=true id_attr="title" name="title" />
                    <Textarea label="Description" required=true id_attr="description" name="description" />
                    <CustomFileInput input_node_ref=file_input_ref label="Thumbnail" name="thumbnail" id_attr="thumbnail" accept="image/*" required=true />

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
