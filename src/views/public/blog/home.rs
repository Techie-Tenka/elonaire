use std::time::Duration;

use detaxine_ui::{
    components::{
        actions::button::{BasicButton, ButtonType},
        content::carousel::Carousel,
        data_display::{badge::Badge, chip::Chip, table::pagination::Pagination},
        feedback::modal::modal::{BasicModal, UseCase},
        forms::{
            input::{InputField, InputFieldType},
            reactive_form::ReactiveForm,
        },
        schemas::props::ColorTemperature,
    },
    utils::{formatters::PipeOption, forms::deserialize_form_with_options},
};
use icondata::{BsArrowLeft, BsSearch, VsSettings};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_icons::Icon;
use leptos_meta::*;
use web_sys::{HtmlDivElement, HtmlFormElement, MouseEvent, SubmitEvent};

use crate::{
    components::molecules::{
        blog::{
            blog_post::BlogPostPreview, blog_section::BlogSection, featured_post::FeaturedPost,
        },
        footer::Footer,
    },
    data::{
        context::{blog::use_blog, ui::use_ui},
        models::graphql::email::SubscriberInput,
        models::graphql::shared::{BlogCategory, BlogStatus, FetchBlogPostsQueryFilters},
    },
    utils::custom_traits::EnumerableEnum,
};

#[component]
pub fn BlogHome() -> impl IntoView {
    let blog_ctx = use_blog();
    let ui_ctx = use_ui();

    let subscription_form_ref = NodeRef::new();
    let search_area_ref = NodeRef::new();
    let (query, set_query) = signal(String::new());
    let (show_overlay, set_show_overlay) = signal(false);
    let (form_is_valid, set_form_is_valid) = signal(false);
    let subscribe_button_is_disabled = Memo::new(move |_| !form_is_valid.get());
    let success_modal_is_open = RwSignal::new(false);

    let featured_posts = move || blog_ctx.featured_posts;
    let other_posts = move || blog_ctx.other_posts;
    let search_results = move || blog_ctx.search_results;

    // Fetch featured + other posts on mount.
    Effect::new(move |_| {
        blog_ctx.fetch_featured_posts();
        blog_ctx.fetch_other_posts(FetchBlogPostsQueryFilters {
            is_featured: Some(false),
            status: Some(BlogStatus::Published),
            ..Default::default()
        });
    });

    // React to a successful newsletter subscription.
    Effect::new(move |prev: Option<u64>| {
        let dirty = blog_ctx.subscription_created_dirty.get();
        if let Some(prev) = prev {
            if dirty != prev {
                if let Some(form) = subscription_form_ref
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

    // Search-as-you-type: dispatch to context whenever query changes.
    Effect::new(move |_| {
        let q = query.get();
        if q.is_empty() {
            set_show_overlay.set(false);
            blog_ctx.clear_search();
            return;
        }
        set_show_overlay.set(true);
        blog_ctx.search_blog_posts(q);
    });

    let handle_search_focus = Callback::new(move |_| {
        if let Some(el) = search_area_ref.get() as Option<HtmlDivElement> {
            el.scroll_into_view_with_bool(true);
            set_show_overlay.set(true);
        }
    });

    let handle_blur = Callback::new(move |_| {
        set_timeout(
            move || {
                set_show_overlay.set(false);
                ui_ctx.show_mobile_search.set(false);
            },
            Duration::from_millis(150),
        );
    });

    let handle_subscribe_form_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        ev.stop_propagation();

        let Some(form) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        else {
            return;
        };

        set_form_is_valid.set(form.check_validity());

        if ev.submitter().is_some() && form_is_valid.get_untracked() {
            let Some(subscriber) = deserialize_form_with_options::<SubscriberInput>(
                &subscription_form_ref,
                &Default::default(),
            ) else {
                return;
            };
            blog_ctx.create_subscription(subscriber);
        }
    };

    let categories = RwSignal::new(
        BlogCategory::variants_slice()
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>(),
    );

    view! {
        <Title text="Blog Home"/>
        <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
            <div class="p-[10px]">
                <p>"You have successfully subscribed to our newsletter! We guarantee that you will only receive updates when there are new publications."</p>
            </div>
        </BasicModal>
        <main>
            <div class="min-h-svh flex flex-col gap-[40px]">
                <div class="display-constraints h-[310px] md:h-[499px]">
                    {
                        move || {
                            let featured_posts_val = featured_posts().get();

                            view! {
                                <Carousel>
                                    {
                                            featured_posts_val
                                                .iter()
                                                .map(|blog_post| {
                                                    view! {
                                                        <FeaturedPost
                                                            thumbnail=blog_post.thumbnail.as_ref().unwrap_or(&String::new()).to_owned()
                                                            title=blog_post.title.as_ref().unwrap_or(&String::new()).to_owned()
                                                            short_description=blog_post.short_description.as_ref().unwrap_or(&String::new()).to_owned()
                                                            author_profile_pic=blog_post.full_author_details.as_ref().unwrap_or(&Default::default()).profile_picture.as_ref().unwrap_or(&String::new()).to_owned()
                                                            author_name=blog_post.full_author_details.as_ref().unwrap_or(&Default::default()).full_name.as_ref().unwrap_or(&String::new()).to_owned() link=blog_post.link.as_ref().unwrap_or(&String::new()).to_owned()
                                                        />
                                                    }.into_any()
                                                })
                                                .collect_view()
                                        }
                                </Carousel>
                            }.into_any()
                        }
                    }

                </div>
                <div class="bg-primary/25 hidden md:block">
                    <div class="display-constraints flex flex-col gap-[20px] items-center justify-center h-[284px]" node_ref=search_area_ref>
                        <h1>"Find just what you are looking for"</h1>
                        <p>"Search through our collection of articles"</p>
                        <div class="w-[384px] flex items-center gap-[10px]">
                            <div class="flex-1 relative">
                                <InputField field_type=InputFieldType::Text icon=BsSearch onfocus=handle_search_focus id_attr="search-input" onblur=handle_blur on:input=move |e| {
                                    set_query.set(event_target_value(&e));
                                } />
                                {move || show_overlay.get().then(|| view! {
                                    <div
                                        class="absolute top-full mt-2 z-50 w-full bg-contrast-white rounded-[5px] shadow-2xl h-[43svh] overflow-y-auto"
                                        on:mousedown=|e: MouseEvent| e.prevent_default()
                                    >
                                        {move || blog_ctx.search_is_loading.get().then(|| view! {
                                            <div class="flex items-center justify-center py-8 text-sm">
                                                <span>"Searching..."</span>
                                            </div>
                                        })}

                                        {move || {
                                            let results = search_results().get();
                                            (!results.is_empty()).then(|| view! {
                                                <ul class="py-2 list-none">
                                                    {results.into_iter().map(|article| view! {
                                                        <li>
                                                            <a
                                                                href=format!("/blog/read/{}", article.link.unwrap_or_default())
                                                                class="flex flex-col px-4 py-3 hover:bg-primary/10 \
                                                                       transition-colors cursor-pointer group"
                                                            >
                                                                <span class="text-sm font-medium \
                                                                             group-hover:text-primary line-clamp-1">
                                                                    {article.title.unwrap_or_default()}
                                                                </span>
                                                                <span class="text-xs mt-0.5">
                                                                    {article.category.text(None)}
                                                                </span>
                                                            </a>
                                                        </li>
                                                    }).collect_view()}
                                                </ul>
                                            })
                                        }}

                                        {move || {
                                            let q = query.get();
                                            let results = search_results().get();
                                            (!q.is_empty() && results.is_empty() && !blog_ctx.search_is_loading.get()).then(|| view! {
                                                <div class="py-8 text-center text-sm">
                                                    "No articles found for "
                                                    <span class="font-medium">{q}</span>
                                                </div>
                                            })
                                        }}
                                    </div>
                                }.into_any())}
                            </div>
                        </div>
                    </div>
                </div>
                {move || ui_ctx.show_mobile_search.get().then(|| view! {
                    <div class="fixed inset-0 z-50 bg-black/50 md:hidden"
                        on:mousedown=move |_| {
                            ui_ctx.show_mobile_search.set(false);
                        }
                    >
                        <div class="bg-contrast-white w-full px-4 py-3 flex items-center gap-3 shadow-lg"
                            on:mousedown=move |e: MouseEvent| e.stop_propagation()
                        >
                            <BasicButton
                                style_ext="shrink-0"
                                on:click=move |_| {
                                    ui_ctx.show_mobile_search.set(false);
                                }
                            >
                                <Icon icon=BsArrowLeft width="1.2rem" height="1.2rem" />
                            </BasicButton>

                            <div class="flex-1 relative">
                                <InputField
                                    field_type=InputFieldType::Text
                                    icon=BsSearch
                                    id_attr="mobile-search-input"
                                    placeholder="Search articles..."
                                    on:input=move |e| {
                                        set_query.set(event_target_value(&e));
                                    }
                                    onblur=handle_blur
                                />
                                {move || {
                                    let q = query.get();
                                    (!q.is_empty()).then(|| view! {
                                        <div class="absolute top-full mt-2 w-full bg-white rounded-[5px] shadow-2xl max-h-[60svh] overflow-y-auto z-50">
                                            {move || blog_ctx.search_is_loading.get().then(|| view! {
                                                <div class="flex items-center justify-center py-8 text-sm">
                                                    <span>"Searching..."</span>
                                                </div>
                                            })}
                                            {move || {
                                                let results = search_results().get();
                                                (!results.is_empty()).then(|| view! {
                                                    <ul class="py-2 list-none">
                                                        {results.into_iter().map(|article| view! {
                                                            <li>
                                                                <a
                                                                    href=format!("/blog/read/{}", article.link.unwrap_or_default())
                                                                    class="flex flex-col px-4 py-3 hover:bg-primary/10 transition-colors cursor-pointer group"
                                                                >
                                                                    <span class="text-sm font-medium group-hover:text-primary line-clamp-1">
                                                                        {article.title.unwrap_or_default()}
                                                                    </span>
                                                                    <span class="text-xs mt-0.5">
                                                                        {article.category.text(None)}
                                                                    </span>
                                                                </a>
                                                            </li>
                                                        }).collect_view()}
                                                    </ul>
                                                })
                                            }}
                                            {move || {
                                                let q = query.get();
                                                let results = search_results().get();
                                                (!q.is_empty() && results.is_empty() && !blog_ctx.search_is_loading.get()).then(|| view! {
                                                    <div class="py-8 text-center text-sm">
                                                        "No articles found for "
                                                        <span class="font-medium">{q}</span>
                                                    </div>
                                                })
                                            }}
                                        </div>
                                    })
                                }}
                            </div>
                        </div>
                    </div>
                })}
                <div class="display-constraints flex flex-col gap-[40px] md:flex-row">
                    <div class="flex flex-col gap-[20px] md:basis-3/4">
                        <div class="flex items-center gap-[10px]">
                            <div class="flex-1">
                                <BlogSection title="More Posts"/>
                            </div>
                            <div class="flex items-center gap-[5px]">
                                <Icon icon=VsSettings width="1rem" height="1rem" />
                                <Badge text="0" ><span>"Filters"</span></Badge>
                            </div>
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-2 md:grid-rows-5 md:auto-rows-min gap-[20px]">
                            { move || {
                                let filtered_posts = other_posts().get();
                                filtered_posts
                                    .iter()
                                    .map(|blog_post| {
                                        view! {
                                            <BlogPostPreview thumbnail=blog_post.thumbnail.as_ref().unwrap_or(&String::new()).to_owned() title=blog_post.title.as_ref().unwrap_or(&String::new()).to_owned() short_description=blog_post.short_description.as_ref().unwrap_or(&String::new()).to_owned() category=blog_post.category.unwrap_or(BlogCategory::Technology).to_owned() read_time=blog_post.read_time.as_ref().unwrap_or(&0).to_owned() link=blog_post.link.as_ref().unwrap_or(&String::new()).to_owned() />
                                        }
                                    })
                                    .collect_view()
                                }
                            }
                        </div>
                        <div class="mt-auto">
                            <Pagination pagination_state=Memo::new(|_| (1, 1)) />
                        </div>
                    </div>
                    <div class="flex flex-col gap-[40px]  md:basis-1/4">
                        <div class="flex flex-col gap-[20px]">
                            <BlogSection title="Stay Updated"/>
                            <div class="flex flex-col gap-[20px]">
                                <p>"Receive Notifications whenever new posts are published. No promotional emails will be sent to your inbox."</p>
                                <ReactiveForm form_ref=subscription_form_ref on:submit=handle_subscribe_form_submit>
                                    <div class="flex flex-col gap-[20px]">
                                        <InputField field_type=InputFieldType::Email label="Email" placeholder="Enter your email" required=true id_attr="email" name="email" />

                                        <BasicButton
                                            button_text="Subscribe"
                                            style_ext="bg-primary text-contrast-white"
                                            button_type=ButtonType::Submit
                                            disabled=subscribe_button_is_disabled
                                        />
                                    </div>
                                </ReactiveForm>
                            </div>
                        </div>
                        <div class="flex flex-col gap-[20px]">
                            <BlogSection title="Categories"/>
                            <div class="flex gap-[16px] flex-wrap">
                                {
                                    move || {
                                        categories.get()
                                            .into_iter()
                                            .map(|category| {
                                                view! {
                                                    <Chip label=category color=ColorTemperature::Gray removable=false />
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                    }
                                }
                            </div>
                        </div>
                    </div>
                </div>
                <div class="mt-auto">
                    <Footer />
                </div>
            </div>
        </main>
    }.into_any()
}
