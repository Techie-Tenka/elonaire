use icondata::BsPlusLg;
use leptos::prelude::*;
use leptos::{ev::SubmitEvent, wasm_bindgen::JsCast};
use leptos_meta::*;
use leptos_router::components::{A, Outlet};
use web_sys::{HtmlFormElement, HtmlInputElement};

use detaxine_ui::{
    components::{
        actions::button::{BasicButton, ButtonType},
        content::richtext_editor::{ExtraFormatingOption, RichTextEditor},
        data_display::table::data_table::{Column, DataTable},
        feedback::{
            modal::modal::{BasicModal, UseCase},
            spinner::Spinner,
        },
        forms::{
            input::{CustomFileInput, InputField, InputFieldType},
            reactive_form::ReactiveForm,
            select::{SelectInput, SelectOption},
            textarea::Textarea,
            toggle_switch::ToggleSwitch,
        },
        navigation::breadcrumbs::Breadcrumbs,
    },
    utils::forms::get_form_data_from_form_ref,
};

use crate::data::{
    context::blog::use_blog,
    models::graphql::shared::{BlogCategory, BlogStatus},
};
use crate::utils::custom_traits::EnumerableEnum;

#[component]
pub fn Blog() -> impl IntoView {
    view! {
        <>
            <Outlet />
        </>
    }
    .into_any()
}

#[component]
pub fn BlogList() -> impl IntoView {
    let table_data = RwSignal::new((
        vec![
            Column::new("Title", false),
            Column::new("Status", true),
            Column::new("Category", true),
            Column::new("Date Created", true),
        ],
        vec![],
    ));

    // TODO: wire up `blog_ctx.fetch_blog_posts(headers, filters, query)` once
    // the selection set is decided, then build the rows from `blog_ctx.blog_posts`.

    view! {
        <>
            <Title text="My Blog Posts"/>
            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Blog Posts"] />
            </div>

            <h1 class="display-constraints">Blog Posts</h1>

            <div class="display-constraints flex items-center justify-end">
                <A href="/dashboard/blog/create">
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
pub fn CreateBlog() -> impl IntoView {
    let form_ref = NodeRef::new();
    let thumbnail_file_input_ref = NodeRef::new();
    let (form_is_valid, set_form_is_valid) = signal(false);
    let submit_is_disabled = Memo::new(move |_| !form_is_valid.get());
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);
    let blog_ctx = use_blog();

    let blog_statuses = RwSignal::new(
        BlogStatus::variants_slice()
            .iter()
            .map(|status| SelectOption::new(&format!("{status:?}"), &status.to_string()))
            .collect::<Vec<SelectOption>>(),
    );

    let blog_categories = RwSignal::new(
        BlogCategory::variants_slice()
            .iter()
            .map(|category| SelectOption::new(&format!("{category:?}"), &category.to_string()))
            .collect::<Vec<SelectOption>>(),
    );

    // Reacts to a successful create: reset the form and show the success modal.
    Effect::new(move |prev: Option<u64>| {
        let dirty = blog_ctx.created_post_dirty.get();
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

        let Some(thumbnail_file_input) =
            thumbnail_file_input_ref.get_untracked() as Option<HtmlInputElement>
        else {
            return;
        };

        let thumbnail_files: Vec<web_sys::File> = thumbnail_file_input
            .files()
            .map(|list| (0..list.length()).filter_map(|i| list.item(i)).collect())
            .unwrap_or_default();

        let Some(form_data) = get_form_data_from_form_ref(&form_ref) else {
            return;
        };

        blog_ctx.create_blog_post(thumbnail_files, form_data);
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
            <Title text="Create Blog Post"/>
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Blog Post created successfully!"</p>
                </div>
            </BasicModal>
            <BasicModal title="Confirm" on_click_primary=onprimary_handler is_open=confirm_modal_is_open use_case=UseCase::Confirmation disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Are you sure that you want to submit?"</p>
                </div>
            </BasicModal>
            <Show when=move || blog_ctx.is_loading.get()>
                <Spinner />
            </Show>

            <div class="display-constraints">
                <Breadcrumbs custom_route_names=["Home", "Dashboard", "Blog Posts", "New"] />
            </div>

            <h1 class="display-constraints">Create New Blog Post</h1>

            <ReactiveForm on:submit=handle_step_form_submit form_ref=form_ref>
                <div class="display-constraints flex flex-col gap-[20px]">
                    <InputField field_type=InputFieldType::Text label="Title" required=true id_attr="title" name="title" />
                    <Textarea label="Short Description" required=true id_attr="short_description" name="short_description" />
                    <SelectInput label="Status" name="status" required=true id_attr="status" placeholder="Select Status" options=blog_statuses />
                    <SelectInput label="Category" name="category" required=true id_attr="category" placeholder="Select Category" options=blog_categories />
                    <ToggleSwitch label_active="Premium" label_inactive="Free" name="is_premium" id_attr="is_premium" initial_active_state=false />
                    <ToggleSwitch label_active="Featured" label_inactive="Not Featured" name="is_featured" id_attr="is_featured" initial_active_state=false />
                    <CustomFileInput input_node_ref=thumbnail_file_input_ref label="Thumbnail" name="thumbnail" id_attr="thumbnail" accept="image/*" required=true />
                    <RichTextEditor name="content" extra_formating_options=vec![ExtraFormatingOption::InlineCode, ExtraFormatingOption::CodeBlock, ExtraFormatingOption::MarkdownUpload, ExtraFormatingOption::ImageUpload, ExtraFormatingOption::Lists, ExtraFormatingOption::Heading] />
                    <BasicButton button_text="Submit" style_ext="bg-primary text-contrast-white" button_type=ButtonType::Submit disabled=submit_is_disabled />
                </div>
            </ReactiveForm>
        </>
    }.into_any()
}
