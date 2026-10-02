use detaxine_ui::{
    components::{
        actions::button::{BasicButton, ButtonType},
        content::richtext_editor::{ExtraFormatingOption, RichTextEditor},
        feedback::{
            modal::modal::{BasicModal, UseCase},
            spinner::Spinner,
        },
        forms::reactive_form::ReactiveForm,
    },
    utils::{formatters::PipeOption, forms::deserialize_form_with_options},
};
use icondata::{
    AiHeartFilled, BiBookmarkRegular, BiShareAltRegular, BsGithub, BsLinkedin, BsTwitterX,
    FaCommentRegular, FaFaceAngryRegular, FaFaceGrinTearsRegular, FaFaceSadTearRegular,
    FaFaceSurpriseRegular, LuThumbsDown, LuThumbsUp, MdiWeb,
};
use js_sys::wasm_bindgen::prelude::Closure;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::{ev, prelude::*};
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use web_sys::{HtmlDivElement, HtmlFormElement, MouseEvent};

use crate::{
    components::molecules::auth::auth_modal::AuthModal,
    data::{
        context::blog::use_blog,
        models::graphql::shared::{
            BlogCommentInput, BlogPost, FetchSingleBlogPostVars, ReactionType,
        },
    },
};
use crate::{
    components::molecules::blog::blog_comment::CommentReactionDetails, data::context::ui::use_ui,
};
use crate::{
    components::molecules::{
        blog::{blog_comment::BlogComment, blog_post_metadata::BlogDetailMetadata},
        footer::Footer,
    },
    views::public::error_handler::ErrorHandler,
};

const SHARED_SERVICE_API: Option<&str> = option_env!("SHARED_SERVICE_API");

#[component]
pub fn BlogPostDetail() -> impl IntoView {
    let blog_ctx = use_blog();
    let params = use_params_map();

    let menu_visible = RwSignal::new(true);
    let comments_ref = NodeRef::new();
    let comment_form_ref = NodeRef::new();
    let (comment_form_is_valid, set_comment_form_is_valid) = signal(false);
    let submit_is_disabled = Memo::new(move |_| !comment_form_is_valid.get());
    let success_modal_is_open = RwSignal::new(false);
    let confirm_modal_is_open = RwSignal::new(false);
    let (show_reactions, set_show_reactions) = signal(false);
    let hover_timer: StoredValue<Option<i32>> = StoredValue::new(None);
    let auth_modal_is_open = RwSignal::new(false);

    let blog_post = move || blog_ctx.current_blog_post;
    let selected_reaction = Memo::new(move |_| {
        blog_post()
            .get()
            .and_then(|bp: BlogPost| bp.current_user_reaction.as_ref().map(|r| r.reaction_type))
    });

    let reactions = vec![
        (ReactionType::Like, "👍"),
        (ReactionType::Dislike, "👎"),
        (ReactionType::Love, "❤️"),
        (ReactionType::Haha, "😂"),
        (ReactionType::Wow, "😮"),
        (ReactionType::Sad, "😢"),
        (ReactionType::Angry, "😡"),
    ];

    let window_scroll_listener = window_event_listener(ev::scroll, move |_| {
        if let Some(comments_el) = comments_ref.get() as Option<HtmlDivElement> {
            let rect = comments_el.get_bounding_client_rect();
            if let Ok(window_height) = window().inner_height() {
                let window_height = window_height.as_f64().unwrap_or(0.0);
                let is_visible = rect.top() < window_height;
                menu_visible.set(!is_visible);
            };
        }
    });

    // Fetch on mount (and whenever the slug changes).
    Effect::new(move |_| {
        if let Some(slug) = params.read().get("slug") {
            blog_ctx.fetch_single_blog_post(FetchSingleBlogPostVars {
                blog_id_or_slug: slug,
            });
        }
    });

    // React to successful comment creation.
    Effect::new(move |prev: Option<u64>| {
        let dirty = blog_ctx.comment_created_dirty.get();
        if let Some(prev) = prev {
            if dirty != prev {
                if let Some(form) = comment_form_ref
                    .get_untracked()
                    .and_then(|el: HtmlFormElement| el.dyn_into::<HtmlFormElement>().ok())
                {
                    form.reset();
                    set_comment_form_is_valid.set(false);
                }
                success_modal_is_open.set(true);
            }
        }
        dirty
    });

    let handle_comment_form_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        if let Some(form) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        {
            set_comment_form_is_valid.set(form.check_validity());
            if ev.submitter().is_some() {
                confirm_modal_is_open.update(|s| *s = true);
            }
        }
    };

    let onprimary_handler = Callback::new(move |_| {
        if !comment_form_is_valid.get() {
            return;
        }
        let Some(blog_post_id) = blog_post().get().and_then(|bp| bp.id) else {
            return;
        };
        let Some(blog_comment) = deserialize_form_with_options::<BlogCommentInput>(
            &comment_form_ref,
            &Default::default(),
        ) else {
            return;
        };

        blog_ctx.create_blog_comment(blog_post_id, blog_comment);
    });

    let handle_scroll_to_comments = Callback::new(move |_| {
        if let Some(el) = comments_ref.get() as Option<HtmlDivElement> {
            el.scroll_into_view_with_bool(true);
        }
    });

    let clear_timer = move || {
        if let Some(id) = hover_timer.get_value() {
            window().clear_timeout_with_handle(id);
            hover_timer.set_value(None);
        }
    };

    let on_mouse_enter = move |_ev: MouseEvent| {
        let id = window()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &Closure::<dyn Fn()>::new(move || {
                    set_show_reactions.set(true);
                })
                .into_js_value()
                .unchecked_ref(),
                1000,
            )
            .unwrap_or_default();
        hover_timer.set_value(Some(id));
    };

    let on_mouse_leave = move |_ev: MouseEvent| {
        clear_timer();
        set_show_reactions.set(false);
    };

    let on_touch_start = move |_| {
        let id = window()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &Closure::<dyn Fn()>::new(move || {
                    set_show_reactions.set(true);
                })
                .into_js_value()
                .unchecked_ref(),
                1000,
            )
            .unwrap_or_default();
        hover_timer.set_value(Some(id));
    };

    let on_touch_end = move |_| {
        // Only clear timer, don't hide reactions — let user pick
        clear_timer();
    };

    let handle_reaction_click = move |reaction: ReactionType| {
        if let Some(blog_post_id) = blog_post().get_untracked().and_then(|bp| bp.id) {
            blog_ctx.react_to_blog_post(blog_post_id, reaction);
        }
    };

    let handle_comment_reaction_click = Callback::new(move |reaction: CommentReactionDetails| {
        blog_ctx.react_to_blog_comment(reaction.comment_id.clone(), reaction.reaction_type.clone());
    });

    let handle_share = Callback::new(move |_| {
        let url = window().location().href().unwrap_or_default();
        let title = blog_post()
            .get()
            .map(|p| p.title.unwrap_or_default())
            .unwrap_or_default();

        let navigator = window().navigator();

        if js_sys::Reflect::has(&navigator, &"share".into()).unwrap_or(false) {
            let share_data = web_sys::ShareData::new();
            share_data.set_url(&url);
            share_data.set_title(&title);

            let promise = navigator.share_with_data(&share_data);
            spawn_local(async move {
                match wasm_bindgen_futures::JsFuture::from(promise).await {
                    Ok(_) => {
                        // The mutation is the only request-shaped step left —
                        // hand it to the context.
                        if let Some(blog_post_id) = blog_ctx
                            .current_blog_post
                            .get_untracked()
                            .and_then(|bp| bp.id)
                        {
                            blog_ctx.update_blog_post_share_count(blog_post_id);
                        }
                    }
                    Err(err) => {
                        let name = js_sys::Reflect::get(&err, &"name".into())
                            .unwrap_or_default()
                            .as_string()
                            .unwrap_or_default();
                        if name != "AbortError" {
                            leptos::logging::error!("Share failed: {:?}", err);
                        }
                    }
                }
            });
        } else {
            let _ = navigator.clipboard().write_text(&url);
        }
    });

    let handle_bookmark = Callback::new(move |_| {
        if let Some(blog_post_id) = blog_post().get_untracked().and_then(|bp| bp.id) {
            blog_ctx.bookmark_blog_post(blog_post_id);
        }
    });

    on_cleanup(move || {
        window_scroll_listener.remove();
    });

    view! {
        <ErrorHandler unauthorized_cb=Callback::new(move |_| auth_modal_is_open.set(true)) />
        <AuthModal is_open=auth_modal_is_open />
        <Title text="Blog Detail"/>

        <main>
            <div class="min-h-svh flex flex-col gap-[40px] relative">
            <BasicModal title="Success" is_open=success_modal_is_open use_case=UseCase::Success disable_auto_close=false>
                <div class="p-[10px]">
                    <p>"Rate Card created successfully!"</p>
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
                {
                    move || {
                        let blog_post = blog_ctx.current_blog_post.get();
                        let selected_reaction_icon = match selected_reaction.get() {
                            Some(ReactionType::Like)    => LuThumbsUp,
                            Some(ReactionType::Dislike) => LuThumbsDown,
                            Some(ReactionType::Love)    => AiHeartFilled,
                            Some(ReactionType::Haha)    => FaFaceGrinTearsRegular,
                            Some(ReactionType::Wow)     => FaFaceSurpriseRegular,
                            Some(ReactionType::Sad)     => FaFaceSadTearRegular,
                            Some(ReactionType::Angry)   => FaFaceAngryRegular,
                            _                        => LuThumbsUp,
                        };

                        if let Some(blog_post) = blog_post {
                            let blog_comments_ref = blog_post.comments.as_ref();

                            Some(
                                view! {
                                    <article class="flex flex-col gap-[20px] display-constraints blog-display-constraints">
                                        <h1>{blog_post.title.as_ref().unwrap_or(&Default::default()).to_owned()}</h1>
                                        <div class="flex items-center">
                                            {
                                                match &blog_post.full_author_details {
                                                    Some(author_details) => {
                                                        Some(
                                                            view! {
                                                                <BlogDetailMetadata date_of_creation={blog_post.created_at.as_ref().unwrap_or(&Default::default()).to_owned()} read_time={blog_post.read_time.as_ref().unwrap_or(&Default::default()).to_owned()} author_profile_pic={author_details.profile_picture.as_ref().unwrap_or(&Default::default()).to_owned()} author_name={author_details.full_name.as_ref().unwrap_or(&Default::default()).to_owned()} />
                                                            }.into_any()
                                                        )
                                                    }
                                                    None => None
                                                }
                                            }
                                        </div>
                                        <img src=format!("{}?width=600", blog_post.thumbnail.as_ref().unwrap_or(&Default::default()).to_owned()) alt="Blog Post Image" class="w-full h-auto rounded-[5px] object-cover" />
                                        // Blog post content goes here
                                        <div class="flex flex-col gap-[20px] leading-relaxed text-lg"  inner_html={blog_post.content.as_ref().unwrap_or(&Default::default()).to_owned()} />
                                    </article>

                                    // floating reaction menu
                                    <div class=move || format!("flex items-center justify-between rounded-full bg-contrast-white border-[1px] border-light-gray absolute fixed bottom-[3%] left-1/2 z-10 -translate-x-1/2 shadow-lg mx-auto transition-all transition-discrete duration-300 h-[47px] {}", if menu_visible.get() {
                                                "translate-y-0 block"
                                            } else {
                                                "translate-y-[-100%] hidden"
                                            })>
                                        // <BasicButton button_text="2K" icon=Some(LuThumbsUp) icon_before=true />
                                        <div
                                            class="relative"
                                            on:mouseenter=on_mouse_enter
                                            on:mouseleave=on_mouse_leave
                                            on:touchstart=on_touch_start
                                            on:touchend=on_touch_end
                                        >
                                            {/* Reactions popup */}
                                            <div class=move || format!(
                                                "absolute bottom-full flex items-center gap-[20px] bg-contrast-white border border-light-gray rounded-full px-3 py-2 shadow-lg transition-all duration-200 {}",
                                                if show_reactions.get() { "opacity-100 translate-y-0 pointer-events-auto" }
                                                else { "opacity-0 translate-y-2 pointer-events-none" }
                                            )>
                                                {reactions.clone().into_iter().map(|(reaction_type, emoji)| {

                                                    let is_selected = move || selected_reaction.get() == Some(reaction_type);
                                                    view! {
                                                        <BasicButton
                                                            style_ext=format!(
                                                                "text-xl transition-transform duration-150 cursor-pointer hover:scale-125 flex flex-col items-center gap-1 {}",
                                                                if is_selected() { "scale-125" } else { "" }
                                                            )
                                                            on:click=move |_| {
                                                                // set_selected_reaction.set(Some(reaction_type));
                                                                set_show_reactions.set(false);
                                                                handle_reaction_click(reaction_type);
                                                            }
                                                        >
                                                            {emoji}
                                                        </BasicButton>
                                                    }.into_any()
                                                }).collect::<Vec<_>>()}
                                            </div>

                                            {/* Like button — updates icon/text based on selected reaction */}
                                            <div on:click=move |_| {
                                                handle_reaction_click(ReactionType::Like);
                                            }>
                                                <BasicButton
                                                    button_text=blog_post.reaction_count.as_ref().map(|count| count.to_string()).unwrap_or_default()
                                                    icon=Some(selected_reaction_icon)
                                                    icon_before=true
                                                    style_ext=format!("{}", if blog_post.current_user_reaction.is_some() { "text-primary" } else { "" })
                                                />
                                            </div>
                                        </div>
                                        <BasicButton button_text=blog_post.bookmarks_count.unwrap_or_default().to_string() icon=Some(BiBookmarkRegular) onclick=handle_bookmark icon_before=true style_ext=format!("{}", if blog_post.current_user_bookmarked.is_some() && blog_post.current_user_bookmarked.unwrap_or_default() { "text-primary" } else { "" }) />
                                        <BasicButton button_text=format!("{}", blog_comments_ref.as_ref().unwrap_or(&&vec![]).len()) icon=Some(FaCommentRegular) onclick=handle_scroll_to_comments icon_before=true />
                                        <BasicButton button_text=blog_post.shares_count.unwrap_or_default().to_string() icon=Some(BiShareAltRegular) icon_before=true onclick=handle_share />
                                    </div>

                                    <div class="flex flex-col gap-[20px] display-constraints blog-display-constraints" node_ref=comments_ref>
                                        <h3>{format!("Comments({})", blog_comments_ref.unwrap_or(&vec![]).len())}</h3>
                                        // WYSIWYG editor goes here
                                        <ReactiveForm on:submit=handle_comment_form_submit form_ref=comment_form_ref>
                                            <div class="flex flex-col gap-[20px]">
                                                <RichTextEditor name="content" extra_formating_options=vec![ExtraFormatingOption::InlineCode, ExtraFormatingOption::CodeBlock] />
                                                <div class="flex gap-[20px] items-center justify-end">
                                                    <BasicButton
                                                        button_text="Submit"
                                                        style_ext="bg-primary text-contrast-white"
                                                        button_type=ButtonType::Submit
                                                        disabled=submit_is_disabled
                                                    />
                                                </div>
                                            </div>
                                        </ReactiveForm>
                                    </div>

                                    <div class="flex flex-col gap-[20px] py-[20px] display-constraints blog-display-constraints border-y border-light-gray">
                                    {
                                        match blog_comments_ref {
                                            Some(comments) => {
                                                Some(
                                                    view! {
                                                        {
                                                            comments.into_iter()
                                                            .map(|comment| {
                                                                view! {
                                                                    <BlogComment comment_id=comment.id.as_ref().unwrap_or(&String::new()).to_owned() content=comment.content.as_ref().unwrap_or(&String::new()).to_owned() date_of_creation=comment.created_at.as_ref().unwrap_or(&String::new()).to_owned() author_name=comment.full_author_details.as_ref().unwrap_or(&Default::default()).full_name.as_ref().unwrap_or(&Default::default()).to_owned() author_avatar=comment.full_author_details.as_ref().unwrap_or(&Default::default()).profile_picture.as_ref().unwrap_or(&Default::default()).to_owned() reply_count=comment.reply_count.as_ref().unwrap_or(&0).to_owned() reaction_count=comment.reaction_count.as_ref().unwrap_or(&0).to_owned() current_user_reaction=comment.current_user_reaction.as_ref().map(|r| r.reaction_type) on_reaction=handle_comment_reaction_click />
                                                                }
                                                            })
                                                            .collect_view()
                                                        }
                                                    }.into_any()
                                                )
                                            }
                                            None => None
                                        }
                                    }
                                    {
                                        match blog_comments_ref {
                                            Some(comments) => {
                                                if comments.is_empty() {
                                                    Some(view! {
                                                        <p class="text-2xl text-center">"No Comments to display"</p>
                                                    }.into_any())
                                                } else {
                                                    None
                                                }
                                            },
                                            None => None
                                        }
                                    }
                                    </div>
                                    <div class="flex flex-col gap-[20px] display-constraints blog-display-constraints">
                                        <h3>Meet the Author</h3>
                                        {
                                            match &blog_post.full_author_details {
                                                Some(author_details) => {
                                                    Some(
                                                        view! {
                                                            <div class="flex py-[20px] gap-[20px] border-y border-light-gray">
                                                                <img src={author_details.profile_picture.as_ref().unwrap_or(&Default::default()).to_owned()} alt="Author Image" class="w-[100px] h-[100px] rounded-full" />
                                                                <div class="flex flex-col gap-[15px]">
                                                                    <p class="font-bold">{author_details.full_name.text(None)}</p>
                                                                    <p>{author_details.bio.text(None)}</p>
                                                                    // Socials
                                                                    <div class="flex gap-[15px]">
                                                                    {
                                                                            author_details.socials.as_ref().map(|socials| {
                                                                                socials.iter().map(|social| {
                                                                                    let (icon, url) = match social.name.to_lowercase().as_str() {
                                                                                        "github" => (BsGithub, social.url.clone()),
                                                                                        "linkedin" => (BsLinkedin, social.url.clone()),
                                                                                        "x" | "twitter" => (BsTwitterX, social.url.clone()),
                                                                                        _ => (MdiWeb, social.url.clone()),
                                                                                    };
                                                                                    view! {
                                                                                        <A href=url attr:class="hover:text-primary transition-colors" target="_blank">
                                                                                            <Icon icon=icon />
                                                                                        </A>
                                                                                    }
                                                                                }).collect::<Vec<_>>()
                                                                            })
                                                                        }
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    )
                                                }
                                                None => None
                                            }
                                        }
                                    </div>
                                }.into_any()
                            )
                        } else {
                            None
                        }
                    }
                }

            </div>
        </main>
        <div class="mt-auto">
            <Footer />
        </div>
    }.into_any()
}
