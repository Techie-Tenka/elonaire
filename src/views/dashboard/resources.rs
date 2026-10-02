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

use crate::data::context::acl::use_acl;
use crate::data::models::graphql::acl::{ResourceInput, ResourceMetadata};

#[component]
pub fn Resources() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn ResourcesList() -> impl IntoView {
    let acl_ctx = use_acl();
    let resources = move || acl_ctx.resources;

    let table_data = RwSignal::new((
        vec![
            Column::new("Resource Name", false),
            Column::new("Date of Creation", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        acl_ctx.fetch_resources();
    });

    Effect::new(move || {
        let resources: Vec<HashMap<String, TableCellData>> = resources()
            .get()
            .iter()
            .map(|resource| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".to_string(),
                    TableCellData::String(
                        resource
                            .id
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Resource Name".to_string(),
                    TableCellData::String(
                        resource
                            .name
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Date of Creation".to_string(),
                    TableCellData::DateTime(
                        resource
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
            prev.1 = resources;
        });
    });

    view! {
        <>
            <Title text="Resources"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Resources"] />
            </div>
            <Show when=move || acl_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Resources</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/resources/create">
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
pub fn CreateResource() -> impl IntoView {
    let form_ref = NodeRef::new();
    let metadata_form_ref = NodeRef::new();
    let (main_form_is_valid, set_main_form_is_valid) = signal(false);
    let (metadata_form_is_valid, set_metadata_form_is_valid) = signal(false);
    let submit_is_disabled =
        Memo::new(move |_| !main_form_is_valid.get() || !metadata_form_is_valid.get());
    let acl_ctx = use_acl();
    let organizations = move || acl_ctx.organizations;
    let departments = move || acl_ctx.departments;
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);
    let departments_options = RwSignal::new(vec![] as Vec<SelectOption>);
    let organizations_options = RwSignal::new(vec![] as Vec<SelectOption>);

    // React to successful creation.
    Effect::new(move |prev: Option<u64>| {
        let dirty = acl_ctx.resource_created_dirty.get();
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

        let Some(resource_input) =
            deserialize_form_with_options::<ResourceInput>(&form_ref, &Default::default())
        else {
            return;
        };
        let Some(resource_metadata) = deserialize_form_with_options::<ResourceMetadata>(
            &metadata_form_ref,
            &Default::default(),
        ) else {
            return;
        };

        acl_ctx.create_resource(resource_input, resource_metadata);
    });

    // Kick off both fetches. `Effect::new` body runs once on mount.
    Effect::new(move |_| {
        acl_ctx.fetch_organizations();
        acl_ctx.fetch_departments();
    });

    // Derive the select options from the context signals.
    Effect::new(move |_| {
        organizations_options.set(
            organizations()
                .get()
                .iter()
                .map(|org| {
                    SelectOption::new(
                        org.id.as_ref().unwrap_or(&Default::default()),
                        org.org_name.as_ref().unwrap_or(&Default::default()),
                    )
                })
                .collect(),
        );

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
            <Title text="New Resource"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Resource created successfully!"</p>
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
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Resources", "New"] />
            </div>

            <h1 class="display-constraints">New Resource</h1>

            <h2 class="display-constraints">Resource Metadata</h2>
            <ReactiveForm on:submit=handle_metadata_form_submit form_ref=metadata_form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <SelectInput
                        label="Organization"
                        name="organization_id"
                        id_attr="organization_id"
                        placeholder="Select Organization"
                        options=organizations_options
                    />
                    <SelectInput
                        label="Department"
                        name="department_id"
                        id_attr="department_id"
                        placeholder="Select Department"
                        options=departments_options
                    />
                </div>
            </ReactiveForm>

            <h2 class="display-constraints">Resource Info</h2>
            <ReactiveForm on:submit=handle_main_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Text label="Resource Name" required=true id_attr="name" name="name" />

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
