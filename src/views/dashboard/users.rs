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
    context::user::use_user,
    models::graphql::acl::{AccountStatus, UserInput},
};

#[component]
pub fn Users() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn UsersList() -> impl IntoView {
    let user_ctx = use_user();
    let users = move || user_ctx.users;

    let table_data = RwSignal::new((
        vec![
            Column::new("Full Name", false),
            Column::new("Email", true),
            Column::new("OAuth Client", true),
            Column::new("Status", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        user_ctx.fetch_users();
    });

    Effect::new(move || {
        let users_rows: Vec<HashMap<String, TableCellData>> = users()
            .get()
            .iter()
            .map(|user| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".to_string(),
                    TableCellData::String(
                        user.id.as_ref().unwrap_or(&Default::default()).to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Full Name".to_string(),
                    TableCellData::String(
                        user.full_name.as_ref().unwrap_or(&String::new()).to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Email".to_string(),
                    TableCellData::String(user.email.to_owned()),
                );

                let oauth_client = match user.oauth_client {
                    Some(client) => format!("{:?}", client),
                    None => String::from("None"),
                };
                hash_map_data.insert(
                    "OAuth Client".to_string(),
                    TableCellData::String(oauth_client),
                );

                let status = match user.status.as_ref().unwrap_or(&AccountStatus::Inactive) {
                    AccountStatus::Active => ViewFn::from(move || {
                        view! { <LabelTag label="Active" color=ColorTemperature::Success /> }
                    }),
                    AccountStatus::Inactive => ViewFn::from(move || {
                        view! { <LabelTag label="InActive" color=ColorTemperature::Info /> }
                    }),
                    AccountStatus::Suspended => ViewFn::from(move || {
                        view! { <LabelTag label="Suspended" color=ColorTemperature::Warning /> }
                    }),
                    AccountStatus::Deleted => ViewFn::from(move || {
                        view! { <LabelTag label="Deleted" color=ColorTemperature::Danger /> }
                    }),
                };
                hash_map_data.insert("Status".to_string(), TableCellData::Html(status));

                hash_map_data
            })
            .collect();

        table_data.update(move |prev| {
            prev.1 = users_rows;
        });
    });

    view! {
        <>
            <Title text="Users"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Users"] />
            </div>
            <Show when=move || user_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Users</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/users/create">
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
pub fn CreateUser() -> impl IntoView {
    let form_ref = NodeRef::new();
    let (form_is_valid, set_form_is_valid) = signal(false);
    let submit_is_disabled = Memo::new(move |_| !form_is_valid.get());
    let user_ctx = use_user();
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);

    // React to successful creation.
    Effect::new(move |prev: Option<u64>| {
        let dirty = user_ctx.user_created_dirty.get();
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

        let Some(user_input) = deserialize_form_with_options::<UserInput>(
            &form_ref,
            &FormDeserializeOptions {
                deserialize_bool: true,
                ..Default::default()
            },
        ) else {
            return;
        };

        user_ctx.create_user(user_input);
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
            <Title text="New User"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"User created successfully!"</p>
                </div>
            </BasicModal>
            <BasicModal title="Confirm" on_click_primary=onprimary_handler is_open=confirm_modal_is_open use_case=UseCase::Confirmation disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Are you sure that you want to submit?"</p>
                </div>
            </BasicModal>
            <Show when=move || user_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Users", "New"] />
            </div>

            <h1 class="display-constraints">New User</h1>

            <ReactiveForm on:submit=handle_step_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Email label="Email" required=true id_attr="email" name="email" />
                    <InputField field_type=InputFieldType::Password label="Password" required=true id_attr="password" name="password" />

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
