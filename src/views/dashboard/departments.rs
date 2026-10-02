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
            select::{SelectInput, SelectOption},
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
    context::acl::use_acl,
    models::graphql::acl::{DepartmentInput, DepartmentMetadata},
};

#[component]
pub fn Departments() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn DepartmentsList() -> impl IntoView {
    let acl_ctx = use_acl();
    let departments = move || acl_ctx.departments;

    let table_data = RwSignal::new((
        vec![
            Column::new("Name", false),
            Column::new("Date of Creation", true),
        ],
        vec![],
    ));

    Effect::new(move || {
        let departments_data: Vec<HashMap<String, TableCellData>> = departments()
            .get()
            .iter()
            .map(|department| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".to_string(),
                    TableCellData::String(
                        department
                            .id
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Name".to_string(),
                    TableCellData::String(
                        department
                            .dep_name
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Date of Creation".to_string(),
                    TableCellData::DateTime(
                        department
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
            prev.1 = departments_data;
        });
    });

    Effect::new(move |_| {
        acl_ctx.fetch_departments();
    });

    view! {
        <>
            <Title text="Departments"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Departments"] />
            </div>
            <Show when=move || acl_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Departments</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/departments/create">
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
pub fn CreateDepartment() -> impl IntoView {
    let form_ref = NodeRef::new();
    let metadata_form_ref = NodeRef::new();
    let (main_form_is_valid, set_main_form_is_valid) = signal(false);
    let (metadata_form_is_valid, set_metadata_form_is_valid) = signal(false);
    let submit_is_disabled =
        Memo::new(move |_| !main_form_is_valid.get() || !metadata_form_is_valid.get());
    let acl_ctx = use_acl();
    let departments = move || acl_ctx.departments;
    let organizations = move || acl_ctx.organizations;
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);
    let departments_options = RwSignal::new(vec![] as Vec<SelectOption>);
    let organizations_options = RwSignal::new(vec![] as Vec<SelectOption>);

    Effect::new(move |prev: Option<u64>| {
        let dirty = acl_ctx.department_created_dirty.get();
        if let Some(prev) = prev {
            if dirty != prev {
                if let Some(form) = form_ref
                    .get_untracked()
                    .and_then(|el: HtmlFormElement| el.dyn_into::<HtmlFormElement>().ok())
                {
                    form.reset();
                    set_main_form_is_valid.set(false);
                }
                if let Some(form) = metadata_form_ref
                    .get_untracked()
                    .and_then(|el: HtmlFormElement| el.dyn_into::<HtmlFormElement>().ok())
                {
                    form.reset();
                    set_metadata_form_is_valid.set(false);
                }
                success_modal_is_open.set(true);
            }
        }
        dirty
    });

    let onprimary_handler = Callback::new(move |_| {
        if !(metadata_form_is_valid.get() && main_form_is_valid.get()) {
            return;
        }

        let Some(department_input) =
            deserialize_form_with_options::<DepartmentInput>(&form_ref, &Default::default())
        else {
            return;
        };
        let Some(department_metadata) = deserialize_form_with_options::<DepartmentMetadata>(
            &metadata_form_ref,
            &Default::default(),
        ) else {
            return;
        };

        acl_ctx.create_department(department_input, department_metadata);
    });

    Effect::new(move |_| {
        acl_ctx.fetch_organizations();
        acl_ctx.fetch_departments();
    });

    Effect::new(move || {
        departments_options.set(
            departments()
                .get()
                .iter()
                .map(|dep| {
                    SelectOption::new(
                        dep.id.as_ref().unwrap_or(&Default::default()),
                        dep.dep_name.as_ref().unwrap_or(&Default::default()),
                    )
                })
                .collect(),
        );

        organizations_options.set(
            organizations()
                .get()
                .iter()
                .map(|org| {
                    SelectOption::new(
                        org.id.as_ref().unwrap_or(&Default::default()).as_str(),
                        org.org_name
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .as_str(),
                    )
                })
                .collect(),
        );
    });

    let handle_metadata_form_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        if let Some(form) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        {
            set_metadata_form_is_valid.set(form.check_validity());
        }
    };

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
            <Title text="New Department"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Department created successfully!"</p>
                </div>
            </BasicModal>
            <BasicModal title="Confirm" on_click_primary=onprimary_handler is_open=confirm_modal_is_open use_case=UseCase::Confirmation disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Are you sure that you want to submit?"</p>
                </div>
            </BasicModal>
            <Show when=move || acl_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Departments", "New"] />
            </div>

            <h1 class="display-constraints">New Department</h1>

            <h2 class="display-constraints">Department Metadata</h2>
            <ReactiveForm on:submit=handle_metadata_form_submit form_ref=metadata_form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                <SelectInput label="Organization" name="organization_id" id_attr="organization_id" placeholder="Select Organization" options=organizations_options />
                <SelectInput label="Department" name="department_id" id_attr="department_id" placeholder="Select Department" options=departments_options />
                </div>
            </ReactiveForm>

            <h2 class="display-constraints">Department Info</h2>
            <ReactiveForm on:submit=handle_main_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Text label="Department Name" required=true id_attr="dep_name" name="dep_name" />
                    <BasicButton button_text="Submit" style_ext="bg-primary text-contrast-white" button_type=ButtonType::Submit disabled=submit_is_disabled />
                </div>
            </ReactiveForm>
        </>
    }.into_any()
}
