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
        },
        navigation::breadcrumbs::Breadcrumbs,
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

use crate::data::{context::acl::use_acl, models::graphql::acl::OrganizationInput};

#[component]
pub fn Organizations() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn OrganizationsList() -> impl IntoView {
    let acl_ctx = use_acl();
    let organizations = move || acl_ctx.organizations;

    let table_data = RwSignal::new((
        vec![
            Column::new("Name", false),
            Column::new("Date of Creation", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        acl_ctx.fetch_organizations();
    });

    Effect::new(move || {
        let orgs: Vec<HashMap<String, TableCellData>> = organizations()
            .get()
            .iter()
            .map(|organization| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".to_string(),
                    TableCellData::String(
                        organization
                            .id
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Name".to_string(),
                    TableCellData::String(
                        organization
                            .org_name
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Date of Creation".to_string(),
                    TableCellData::DateTime(
                        organization
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
            prev.1 = orgs;
        });
    });

    view! {
        <>
            <Title text="Organizations"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Organizations"] />
            </div>
            <Show when=move || acl_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Organizations</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/organizations/create">
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
pub fn CreateOrganization() -> impl IntoView {
    let form_ref = NodeRef::new();
    let (main_form_is_valid, set_main_form_is_valid) = signal(false);
    let submit_is_disabled = Memo::new(move |_| !main_form_is_valid.get());
    let acl_ctx = use_acl();
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);

    Effect::new(move |prev: Option<u64>| {
        let dirty = acl_ctx.organization_created_dirty.get();
        if let Some(prev) = prev {
            if dirty != prev {
                if let Some(form) = form_ref
                    .get_untracked()
                    .and_then(|el: HtmlFormElement| el.dyn_into::<HtmlFormElement>().ok())
                {
                    form.reset();
                    set_main_form_is_valid.set(false);
                }
                success_modal_is_open.set(true);
            }
        }
        dirty
    });

    let onprimary_handler = Callback::new(move |_| {
        if !main_form_is_valid.get() {
            return;
        }

        let Some(organization_input) = deserialize_form_with_options::<OrganizationInput>(
            &form_ref,
            &FormDeserializeOptions::default(),
        ) else {
            return;
        };

        acl_ctx.create_organization(organization_input);
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
            <Title text="New Organization"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Organization created successfully!"</p>
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
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Organizations", "New"] />
            </div>

            <h1 class="display-constraints">New Organization</h1>

            <ReactiveForm on:submit=handle_main_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Text label="Organization Name" required=true id_attr="org_name" name="org_name" />
                    <BasicButton button_text="Submit" style_ext="bg-primary text-contrast-white" button_type=ButtonType::Submit disabled=submit_is_disabled />
                </div>
            </ReactiveForm>
        </>
    }.into_any()
}
