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
            input::{InputField, InputFieldType},
            reactive_form::ReactiveForm,
            select::{SelectInput, SelectOption},
        },
        navigation::breadcrumbs::Breadcrumbs,
        schemas::props::ColorTemperature,
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
use crate::data::models::graphql::acl::{AdminPrivilege, PermissionInput, PermissionMetadata};
use crate::utils::custom_traits::EnumerableEnum;

#[component]
pub fn Permissions() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn PermissionsList() -> impl IntoView {
    let acl_ctx = use_acl();
    let permissions = move || acl_ctx.permissions;

    let table_data = RwSignal::new((
        vec![
            Column::new("Name", false),
            Column::new("Privilege", true),
            Column::new("Resource", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        acl_ctx.fetch_permissions();
    });

    Effect::new(move || {
        let permissions: Vec<HashMap<String, TableCellData>> = permissions()
            .get()
            .iter()
            .map(|permission| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".to_string(),
                    TableCellData::String(
                        permission
                            .id
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Name".to_string(),
                    TableCellData::String(
                        permission
                            .name
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Resource".to_string(),
                    TableCellData::String(
                        permission
                            .resource
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .name
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                let privilege = if permission.is_admin.is_some()
                    && permission.is_admin.unwrap_or_default()
                {
                    ViewFn::from(move || {
                        view! { <LabelTag label="Admin" color=ColorTemperature::Warning /> }
                    })
                } else if permission.is_super_admin.is_some()
                    && permission.is_super_admin.unwrap_or_default()
                {
                    ViewFn::from(move || {
                        view! { <LabelTag label="Super Admin" color=ColorTemperature::Danger /> }
                    })
                } else {
                    ViewFn::from(move || view! { <LabelTag label="None" /> })
                };
                hash_map_data.insert("Privilege".to_string(), TableCellData::Html(privilege));
                hash_map_data
            })
            .collect();

        table_data.update(move |prev| {
            prev.1 = permissions;
        });
    });

    view! {
        <>
            <Title text="Permissions"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Permissions"] />
            </div>
            <Show when=move || acl_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Permissions</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/permissions/create">
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
pub fn CreatePermission() -> impl IntoView {
    let form_ref = NodeRef::new();
    let metadata_form_ref = NodeRef::new();
    let (main_form_is_valid, set_main_form_is_valid) = signal(false);
    let (metadata_form_is_valid, set_metadata_form_is_valid) = signal(false);
    let submit_is_disabled =
        Memo::new(move |_| !main_form_is_valid.get() || !metadata_form_is_valid.get());
    let acl_ctx = use_acl();
    let resources = move || acl_ctx.resources;
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);
    let resources_options = RwSignal::new(vec![] as Vec<SelectOption>);

    let admin_privileges = RwSignal::new(
        AdminPrivilege::variants_slice()
            .iter()
            .map(|admin_privilege| {
                SelectOption::new(
                    &format!("{admin_privilege:?}"),
                    &admin_privilege.to_string(),
                )
            })
            .collect::<Vec<SelectOption>>(),
    );

    // React to successful creation.
    Effect::new(move |prev: Option<u64>| {
        let dirty = acl_ctx.permission_created_dirty.get();
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

        let Some(permission_input) =
            deserialize_form_with_options::<PermissionInput>(&form_ref, &Default::default())
        else {
            return;
        };
        let Some(permission_metadata) = deserialize_form_with_options::<PermissionMetadata>(
            &metadata_form_ref,
            &Default::default(),
        ) else {
            return;
        };

        acl_ctx.create_permission(permission_input, permission_metadata);
    });

    Effect::new(move |_| {
        acl_ctx.fetch_resources();
    });

    Effect::new(move || {
        resources_options.set(
            resources()
                .get()
                .iter()
                .map(|resources| {
                    SelectOption::new(
                        resources.id.as_ref().unwrap_or(&Default::default()),
                        resources.name.as_ref().unwrap_or(&Default::default()),
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
            <Title text="New Permission"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Permission created successfully!"</p>
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
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Permissions", "New"] />
            </div>

            <h1 class="display-constraints">New Permission</h1>

            <h2 class="display-constraints">Permission Metadata</h2>
            <ReactiveForm on:submit=handle_metadata_form_submit form_ref=metadata_form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <SelectInput
                        label="Resource"
                        name="resource_id"
                        required=true
                        id_attr="resource_id"
                        placeholder="Select Resource"
                        options=resources_options
                    />
                    <SelectInput
                        label="Admin Privilege"
                        name="admin_privilege"
                        required=true
                        id_attr="admin_privilege"
                        placeholder="Select Admin Privilege"
                        options=admin_privileges
                    />
                </div>
            </ReactiveForm>

            <h2 class="display-constraints">Permission Info</h2>
            <ReactiveForm on:submit=handle_main_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Text label="Permission Name" required=true id_attr="name" name="name" />

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
