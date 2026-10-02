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
            select::{CustomSelectInput, SelectInput, SelectOption},
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
    context::portfolio::use_portfolio, models::graphql::shared::UserPortfolioCategory,
};
use crate::utils::custom_traits::EnumerableEnum;

#[component]
pub fn Portfolio() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn PortfolioList() -> impl IntoView {
    let portfolio_ctx = use_portfolio();
    let portfolio = move || portfolio_ctx.portfolio;

    let table_data = RwSignal::new((
        vec![
            Column::new("Title", false),
            Column::new("Start Date", true),
            Column::new("YOE", true),
            Column::new("Category", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        portfolio_ctx.fetch_portfolio();
    });

    Effect::new(move || {
        let portfolio_data: Vec<HashMap<String, TableCellData>> = portfolio()
            .get()
            .iter()
            .map(|portfolio| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".to_string(),
                    TableCellData::String(
                        portfolio
                            .id
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Title".to_string(),
                    TableCellData::String(
                        portfolio
                            .title
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "YOE".to_string(),
                    TableCellData::Usize(
                        portfolio
                            .years_of_experience
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Start Date".to_string(),
                    TableCellData::DateTime(
                        portfolio
                            .start_date
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Category".to_string(),
                    TableCellData::String(format!(
                        "{:?}",
                        portfolio
                            .category
                            .as_ref()
                            .unwrap_or(&UserPortfolioCategory::Rust)
                            .to_owned()
                    )),
                );
                hash_map_data
            })
            .collect();

        table_data.update(move |prev| {
            prev.1 = portfolio_data;
        });
    });

    view! {
        <>
            <Title text="My Portfolio"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Portfolio"] />
            </div>
            <Show when=move || portfolio_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Portfolio</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/portfolio/create">
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
pub fn CreatePortfolio() -> impl IntoView {
    let form_ref = NodeRef::new();
    let file_input_ref = NodeRef::new();
    let applied_skills = RwSignal::new(Vec::new() as Vec<String>);
    let (form_is_valid, set_form_is_valid) = signal(false);
    let submit_is_disabled =
        Memo::new(move |_| !form_is_valid.get() || applied_skills.get().is_empty());
    let portfolio_ctx = use_portfolio();
    let skills = move || portfolio_ctx.skills;
    let skills_select_options = RwSignal::new(vec![] as Vec<SelectOption>);
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);

    let portfolio_categories = RwSignal::new(
        UserPortfolioCategory::variants_slice()
            .iter()
            .map(|category| SelectOption::new(&format!("{category:?}"), &category.to_string()))
            .collect::<Vec<SelectOption>>(),
    );

    // React to successful creation.
    Effect::new(move |prev: Option<u64>| {
        let dirty = portfolio_ctx.portfolio_item_created_dirty.get();
        if let Some(prev) = prev {
            if dirty != prev {
                if let Some(form) = form_ref
                    .get_untracked()
                    .and_then(|el: HtmlFormElement| el.dyn_into::<HtmlFormElement>().ok())
                {
                    form.reset();
                    set_form_is_valid.set(false);
                }
                applied_skills.set(Vec::new());
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

        portfolio_ctx.create_portfolio_item(files, form_data, applied_skills.get_untracked());
    });

    // Derive skill select options.
    Effect::new(move |_| {
        skills_select_options.set(
            skills()
                .get()
                .iter()
                .map(|skill| SelectOption {
                    value: skill.id.as_ref().unwrap_or(&Default::default()).clone(),
                    label: skill.name.as_ref().unwrap_or(&Default::default()).clone(),
                })
                .collect::<Vec<_>>(),
        );
    });

    // Fetch skills for the dropdown on mount.
    Effect::new(move |_| {
        portfolio_ctx.fetch_skills();
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
            <Title text="Create Portfolio"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Portfolio Project created successfully!"</p>
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
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Portfolio", "New"] />
            </div>

            <h1 class="display-constraints">Create New Portfolio Project</h1>

            <ReactiveForm on:submit=handle_step_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Text label="Title" required=true id_attr="title" name="title" />
                    <Textarea label="Description" required=true id_attr="description" name="description" />
                    <DatePicker label="Start Date" required=true id_attr="start_date" name="start_date" />
                    <DatePicker label="End Date" required=true id_attr="end_date" name="end_date" />
                    <InputField field_type=InputFieldType::Text label="Link" required=true id_attr="link" name="link" />
                    <SelectInput
                        label="Category"
                        name="category"
                        required=true
                        id_attr="category"
                        placeholder="Select Category"
                        options=portfolio_categories
                    />
                    <CustomFileInput input_node_ref=file_input_ref label="Thumbnail" name="thumbnail" id_attr="thumbnail" accept="image/*" required=true />
                    <div class="flex flex-col gap-[10px]">
                        <h3>Applied Skills</h3>
                        <div class="flex flex-row items-center">
                            <CustomSelectInput
                                label="Skills"
                                id_attr="skills"
                                multiple=true
                                required=true
                                options=skills_select_options
                                value=applied_skills
                            />
                        </div>
                    </div>
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
