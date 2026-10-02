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
            checkbox::{CheckboxGroup, CheckboxOption},
            input::{InputField, InputFieldType},
            reactive_form::ReactiveForm,
            select::{SelectInput, SelectOption},
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

use crate::data::context::acl::use_acl;
use crate::data::models::graphql::acl::{AdminPrivilege, RoleInput, RoleMetadata};
use crate::utils::custom_traits::EnumerableEnum;

#[component]
pub fn Roles() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn RolesList() -> impl IntoView {
    let acl_ctx = use_acl();
    let roles = move || acl_ctx.roles;

    let table_data = RwSignal::new((
        vec![
            Column::new("Role Name", false),
            Column::new("Privilege", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        acl_ctx.fetch_roles();
    });

    Effect::new(move || {
        let roles: Vec<HashMap<String, TableCellData>> = roles()
            .get()
            .iter()
            .map(|role| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".to_string(),
                    TableCellData::String(
                        role.id.as_ref().unwrap_or(&Default::default()).to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Role Name".to_string(),
                    TableCellData::String(
                        role.role_name
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                let privilege = if role.is_admin.is_some()
                    && role.is_admin.unwrap_or(Default::default())
                {
                    ViewFn::from(move || {
                        view! { <LabelTag label="Admin" color=ColorTemperature::Warning /> }
                    })
                } else if role.is_super_admin.is_some()
                    && role.is_super_admin.unwrap_or(Default::default())
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
            prev.1 = roles;
        });
    });

    view! {
        <>
            <Title text="Roles"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Roles"] />
            </div>
            <Show when=move || acl_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Roles</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/roles/create">
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
pub fn CreateRole() -> impl IntoView {
    let form_ref = NodeRef::new();
    let metadata_form_ref = NodeRef::new();
    let (main_form_is_valid, set_main_form_is_valid) = signal(false);
    let (metadata_form_is_valid, set_metadata_form_is_valid) = signal(false);
    let submit_is_disabled =
        Memo::new(move |_| !main_form_is_valid.get() || !metadata_form_is_valid.get());
    let acl_ctx = use_acl();
    let departments = move || acl_ctx.departments;
    let organizations = move || acl_ctx.organizations;
    let permissions = move || acl_ctx.permissions;
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);
    let departments_options = RwSignal::new(vec![] as Vec<SelectOption>);
    let organizations_options = RwSignal::new(vec![] as Vec<SelectOption>);
    let permissions_options = RwSignal::new(vec![] as Vec<CheckboxOption>);

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
        let dirty = acl_ctx.system_role_created_dirty.get();
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

        let Some(role_input) =
            deserialize_form_with_options::<RoleInput>(&form_ref, &Default::default())
        else {
            return;
        };
        let Some(role_metadata) = deserialize_form_with_options::<RoleMetadata>(
            &metadata_form_ref,
            &FormDeserializeOptions {
                vec_fields: Some(&["permission_ids"]),
                ..Default::default()
            },
        ) else {
            return;
        };

        acl_ctx.create_system_role(role_input, role_metadata);
    });

    // Kick off all three fetches on mount.
    Effect::new(move |_| {
        acl_ctx.fetch_organizations();
        acl_ctx.fetch_departments();
        acl_ctx.fetch_permissions();
    });

    // Derive select/checkbox options from the context signals.
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

        permissions_options.set(
            permissions()
                .get()
                .iter()
                .map(|permission| {
                    CheckboxOption::new(
                        permission.id.as_ref().unwrap_or(&Default::default()),
                        permission.name.as_ref().unwrap_or(&Default::default()),
                        None,
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
            <Title text="New Role"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Role created successfully!"</p>
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
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Roles", "New"] />
            </div>

            <h1 class="display-constraints">New Role</h1>

            <h2 class="display-constraints">Role Metadata</h2>
            <ReactiveForm on:submit=handle_metadata_form_submit form_ref=metadata_form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <SelectInput
                        label="Admin Privilege"
                        name="admin_privilege"
                        required=true
                        id_attr="admin_privilege"
                        placeholder="Select Admin Privilege"
                        options=admin_privileges
                    />
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
                    <CheckboxGroup
                        legend="Permissions"
                        name="permission_ids"
                        options=permissions_options
                    />
                </div>
            </ReactiveForm>

            <h2 class="display-constraints">Role Info</h2>
            <ReactiveForm on:submit=handle_main_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Text label="Role Name" required=true id_attr="role_name" name="role_name" />

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
