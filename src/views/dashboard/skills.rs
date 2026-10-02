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
            datepicker::DatePicker,
            input::{CustomFileInput, InputField, InputFieldType},
            reactive_form::ReactiveForm,
            select::{SelectInput, SelectOption},
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

use crate::data::{
    context::portfolio::use_portfolio,
    models::graphql::shared::{UserSkillLevel, UserSkillType},
};
use crate::utils::custom_traits::EnumerableEnum;

#[component]
pub fn Skills() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn SkillsList() -> impl IntoView {
    let portfolio_ctx = use_portfolio();
    let skills = move || portfolio_ctx.skills;

    let table_data = RwSignal::new((
        vec![
            Column::new("Name", false),
            Column::new("Type", true),
            Column::new("Level", true),
            Column::new("Start Date", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        portfolio_ctx.fetch_skills();
    });

    Effect::new(move || {
        let skills: Vec<HashMap<String, TableCellData>> = skills()
            .get()
            .iter()
            .map(|skill| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".to_string(),
                    TableCellData::String(
                        skill.id.as_ref().unwrap_or(&Default::default()).to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Name".to_string(),
                    TableCellData::String(
                        skill
                            .name
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "YOE".to_string(),
                    TableCellData::Usize(
                        skill
                            .years_of_experience
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Start Date".to_string(),
                    TableCellData::DateTime(
                        skill
                            .start_date
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Level".to_string(),
                    TableCellData::String(format!(
                        "{:?}",
                        skill
                            .level
                            .as_ref()
                            .unwrap_or(&UserSkillLevel::Beginner)
                            .to_owned()
                    )),
                );
                hash_map_data.insert(
                    "Type".to_string(),
                    TableCellData::String(format!(
                        "{:?}",
                        skill
                            .skill_type
                            .as_ref()
                            .unwrap_or(&UserSkillType::Technical)
                            .to_owned()
                    )),
                );
                hash_map_data
            })
            .collect();

        table_data.update(move |prev| {
            prev.1 = skills;
        });
    });

    view! {
        <>
            <Title text="My Skills"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Skills"] />
            </div>
            <Show when=move || portfolio_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">My Skills</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/skills/create">
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
pub fn CreateSkill() -> impl IntoView {
    let form_ref = NodeRef::new();
    let file_input_ref = NodeRef::new();
    let (form_is_valid, set_form_is_valid) = signal(false);
    let submit_is_disabled = Memo::new(move |_| !form_is_valid.get());
    let portfolio_ctx = use_portfolio();
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);

    let user_skill_levels = RwSignal::new(
        UserSkillLevel::variants_slice()
            .iter()
            .map(|level| SelectOption::new(&format!("{level:?}"), &level.to_string()))
            .collect::<Vec<SelectOption>>(),
    );
    let user_skill_types = RwSignal::new(
        UserSkillType::variants_slice()
            .iter()
            .map(|skill_type| {
                SelectOption::new(&format!("{skill_type:?}"), &skill_type.to_string())
            })
            .collect::<Vec<SelectOption>>(),
    );

    // React to successful creation.
    Effect::new(move |prev: Option<u64>| {
        let dirty = portfolio_ctx.skill_created_dirty.get();
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

        portfolio_ctx.create_skill(files, form_data);
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
            <Title text="Create Skill"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Skill created successfully!"</p>
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
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Skills", "New"] />
            </div>

            <h1 class="display-constraints">Create New Skill</h1>

            <ReactiveForm on:submit=handle_step_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Text label="Name" required=true id_attr="name" name="name" />
                    <Textarea label="Description" required=true id_attr="description" name="description" />
                    <SelectInput
                        label="Type"
                        name="skill_type"
                        required=true
                        id_attr="skill_type"
                        placeholder="Select Type"
                        options=user_skill_types
                    />
                    <SelectInput
                        label="Level"
                        name="level"
                        required=true
                        id_attr="level"
                        placeholder="Select Level"
                        options=user_skill_levels
                    />
                    <DatePicker label="Start Date" required=true id_attr="start_date" name="start_date" />
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
