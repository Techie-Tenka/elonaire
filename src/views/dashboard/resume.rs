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
            input::{InputField, InputFieldType},
            reactive_form::ReactiveForm,
            select::{SelectInput, SelectOption},
        },
        navigation::breadcrumbs::Breadcrumbs,
    },
    utils::forms::{FormDeserializeOptions, deserialize_form_with_options},
};
use icondata::{BsPlusLg, TbAwardOffOutline};
use leptos::ev::{self, SubmitEvent};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::components::{A, Outlet};
use web_sys::{HtmlFormElement, HtmlInputElement};

use crate::data::{
    context::portfolio::use_portfolio,
    models::graphql::shared::{UserResumeInput, UserResumeSection},
};
use crate::utils::custom_traits::EnumerableEnum;

#[component]
pub fn Resume() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn ResumeItemsList() -> impl IntoView {
    let portfolio_ctx = use_portfolio();
    let resume = move || portfolio_ctx.resume;

    let table_data = RwSignal::new((
        vec![
            Column::new("Title", false),
            Column::new("Start Date", true),
            Column::new("YOE", true),
            Column::new("Section", true),
        ],
        vec![],
    ));

    Effect::new(move |_| {
        portfolio_ctx.fetch_resume();
    });

    Effect::new(move || {
        let resume_rows: Vec<HashMap<String, TableCellData>> = resume()
            .get()
            .iter()
            .map(|resume| {
                let mut hash_map_data = HashMap::new();

                hash_map_data.insert(
                    "id".to_string(),
                    TableCellData::String(
                        resume.id.as_ref().unwrap_or(&Default::default()).to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Title".to_string(),
                    TableCellData::String(
                        resume
                            .title
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "YOE".to_string(),
                    TableCellData::Usize(
                        resume
                            .years_of_experience
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Start Date".to_string(),
                    TableCellData::DateTime(
                        resume
                            .start_date
                            .as_ref()
                            .unwrap_or(&Default::default())
                            .to_owned(),
                    ),
                );
                hash_map_data.insert(
                    "Section".to_string(),
                    TableCellData::String(format!(
                        "{:?}",
                        resume
                            .section
                            .as_ref()
                            .unwrap_or(&UserResumeSection::Experience)
                            .to_owned()
                    )),
                );
                hash_map_data
            })
            .collect();

        table_data.update(move |prev| {
            prev.1 = resume_rows;
        });
    });

    view! {
        <>
            <Title text="Resume Items"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Resume Items"] />
            </div>
            <Show when=move || portfolio_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <h1 class="display-constraints">Resume Items</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/resume/create">
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
pub fn CreateResumeItem() -> impl IntoView {
    let form_ref = NodeRef::new();
    let (achievement_field_value, set_achievement_field_value) = signal(String::new());
    let add_is_disabled = Memo::new(move |_| !(achievement_field_value.get().len() > 10));
    let (achievements, set_achievements) = signal(Vec::new() as Vec<String>);
    let (form_is_valid, set_form_is_valid) = signal(false);
    let submit_is_disabled =
        Memo::new(move |_| !form_is_valid.get() || achievements.get().len() == 0);
    let portfolio_ctx = use_portfolio();
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);

    let resume_sections = RwSignal::new(
        UserResumeSection::variants_slice()
            .iter()
            .map(|section| SelectOption::new(&format!("{section:?}"), &section.to_string()))
            .collect::<Vec<SelectOption>>(),
    );

    // React to successful creation.
    Effect::new(move |prev: Option<u64>| {
        let dirty = portfolio_ctx.resume_item_created_dirty.get();
        if let Some(prev) = prev {
            if dirty != prev {
                if let Some(form) = form_ref
                    .get_untracked()
                    .and_then(|el: HtmlFormElement| el.dyn_into::<HtmlFormElement>().ok())
                {
                    form.reset();
                    set_form_is_valid.set(false);
                }
                set_achievements.set(vec![]);
                success_modal_is_open.set(true);
            }
        }
        dirty
    });

    let onprimary_handler = Callback::new(move |_| {
        if !form_is_valid.get() {
            return;
        }

        let Some(resume_item) = deserialize_form_with_options::<UserResumeInput>(
            &form_ref,
            &FormDeserializeOptions {
                deserialize_bool: true,
                ..Default::default()
            },
        ) else {
            return;
        };

        portfolio_ctx.create_resume_item(resume_item, achievements.get_untracked());
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

    let handle_achievement_input_change = move |ev: ev::Event| {
        if let Some(input_el) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            set_achievement_field_value.set(input_el.value());
        }
    };

    let handle_add_button_click = Callback::new(move |_ev: ev::MouseEvent| {
        set_achievements.update(|prev| {
            prev.push(achievement_field_value.get());
            set_achievement_field_value.set(String::new());
        });
    });

    view! {
        <>
            <Title text="New Resume Item"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Resume Item created successfully!"</p>
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
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Resume Items", "New"] />
            </div>

            <h1 class="display-constraints">New Resume Item</h1>

            <ReactiveForm on:submit=handle_step_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Text label="Title" required=true id_attr="title" name="title" />
                    <InputField field_type=InputFieldType::Text label="More Info" id_attr="more_info" name="more_info" />

                    <DatePicker label="Start Date" required=true id_attr="start_date" name="start_date" />
                    <DatePicker label="End Date" id_attr="end_date" name="end_date" />
                    <InputField field_type=InputFieldType::Text label="Link" id_attr="link" name="link" />
                    <SelectInput
                        label="Section"
                        name="section"
                        required=true
                        id_attr="section"
                        placeholder="Select Section"
                        options=resume_sections
                    />

                    <div class="flex flex-col gap-[10px]">
                        <h3>Achievements<span class="text-danger">"*"</span></h3>
                        { move || if achievements.get().is_empty() {
                            Some(view! {
                                <div class="flex flex-col">
                                    <Icon icon=TbAwardOffOutline />
                                    <p class="text-sm">No achievements added yet.</p>
                                </div>
                            })
                        } else {
                            None
                        }
                        }
                        <ul class="list-disc list-inside">
                            {
                                move || achievements.get().iter().map(|achievement| view! { <li>{achievement.to_owned()}</li> }).collect::<Vec<_>>()
                            }
                        </ul>
                        <div class="flex flex-row items-center">
                            <InputField field_type=InputFieldType::Text placeholder="Add Achievement" initial_value=achievement_field_value on:input=handle_achievement_input_change id_attr="achievement" ext_wrapper_styles="flex-1" ext_input_styles="rounded-r-none" />
                            <BasicButton
                                button_text="Add"
                                style_ext="bg-primary text-contrast-white rounded-l-none"
                                button_type=ButtonType::Button
                                disabled=add_is_disabled
                                onclick=handle_add_button_click
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
