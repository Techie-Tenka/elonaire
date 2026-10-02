use icondata::{
    BsInfoCircle, BsMoon, BsPower, BsRss, BsSearch, BsSun, CgMenuLeft,
    MdiCardAccountDetailsOutline, MdiCertificateOutline, MdiStore, MdiTabletDashboard,
    MdiTrophyAward, RiArticleDocumentLine,
};
use leptos::ev;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use leptos_router::hooks::{use_location, use_navigate};

use detaxine_ui::components::{
    actions::button::BasicButton, feedback::popover::Popover, forms::toggle_switch::ToggleSwitch,
};

use crate::components::hocs::permission_guard::{PermissionGuard, PermissionMatch};
use crate::data::context::{auth::use_auth, ui::use_ui, user::use_user};
use crate::views::dashboard::layout::MenuItem;

#[component]
pub fn Nav(
    #[prop(optional, default = Callback::new(|_| {}))] onmenuclick: Callback<ev::MouseEvent>,
) -> impl IntoView {
    let location = use_location();
    let showing_user_popover = RwSignal::new(false);
    let navigate = use_navigate();

    let user_ctx = use_user();
    let auth_ctx = use_auth();
    let ui_ctx = use_ui();

    let user_profile = user_ctx.user_profile;
    let dark_mode_is_active = ui_ctx.dark_mode_is_active;

    let is_dashboard = Memo::new(move |_| location.pathname.get().contains("/dashboard"));
    let is_blog = Memo::new(move |_| location.pathname.get().contains("/blog"));
    let is_blog_home = Memo::new(move |_| location.pathname.get() == "/blog");

    let menu_items = Memo::new(move |_| {
        vec![
            MenuItem::new("About", BsInfoCircle, "/about", vec![]),
            MenuItem::new("Resume", MdiCertificateOutline, "/resume", vec![]),
            MenuItem::new("Portfolio", MdiTrophyAward, "/portfolio", vec![]),
            MenuItem::new("Marketplace", MdiStore, "/marketplace", vec![]),
            MenuItem::new("Blog", RiArticleDocumentLine, "/blog", vec![]),
        ]
    });

    let blog_menu_items = Memo::new(move |_| {
        vec![
            MenuItem::new("Blog Feed", BsRss, "/blog", vec![]),
            MenuItem::new("About", BsInfoCircle, "/blog/about", vec![]),
        ]
    });

    // Trigger sign-out — the context handles the rest.
    let handle_sign_out = Callback::new(move |_| {
        auth_ctx.sign_out();
    });

    // React to successful sign-out: clear the user profile context and navigate.
    Effect::new(move |prev: Option<u64>| {
        let dirty = auth_ctx.signed_out_dirty.get();
        if let Some(prev) = prev {
            if dirty != prev {
                user_ctx.clear();
                navigate("/sign-in", Default::default());
            }
        }
        dirty
    });

    // Dark-mode class toggle — pure DOM side effect, stays in the view.
    Effect::new(move |_| {
        if let Some(doc) = document().document_element() {
            let class_list = doc.class_list();

            if !dark_mode_is_active.get() && class_list.contains("dark") {
                let _ = class_list.remove_1("dark");
            } else if !dark_mode_is_active.get() && !class_list.contains("dark") {
            } else {
                let _ = class_list.add_1("dark");
            }
        }
    });

    view! {
        <>
        <div class="border-b-[1px] border-light-gray dark:border-mid-gray">
            <div class="display-constraints flex items-center justify-between h-[47px]">

                // Left — hamburger (mobile only on non-dashboard, always on dashboard)
                {
                    move || {
                        let hidden = if !is_dashboard.get() { "md:hidden" } else { "" };

                        view! {
                            <BasicButton
                                style_ext=format!("border-none cursor-pointer shrink-0 {}", hidden)
                                on:click=move |ev| onmenuclick.run(ev)
                            >
                                <Icon width="24" height="24" icon=CgMenuLeft />
                            </BasicButton>
                        }
                    }
                }

                // Center/Left — logo
                <A href="/" attr:class="flex items-center h-full shrink-0">
                    <img src="https://techietenka.com/api/files/view/default/Techie Tenka-01.png" class="h-full w-auto object-cover" alt="Logo" />
                </A>

                // Right — all right-side items
                <div class="flex items-center gap-[20px]">

                    // Desktop menu items — public
                    {move || (!is_dashboard.get() && !is_blog.get()).then(|| view! {
                        <div class="hidden md:flex items-center gap-[20px] text-sm">
                            <For
                                each=move || menu_items.get()
                                key=|item| item.path.to_owned()
                                let(child)
                            >
                                <A attr:class="flex items-center cursor-pointer hover:text-primary" href=child.path>{child.label}</A>
                            </For>
                        </div>
                    })}

                    // Desktop menu items — blog
                    {move || is_blog.get().then(|| view! {
                        <div class="hidden md:flex items-center gap-[20px] text-sm">
                            <For
                                each=move || blog_menu_items.get()
                                key=|item| item.path.to_owned()
                                let(child)
                            >
                                <A attr:class="flex items-center cursor-pointer hover:text-primary" href=child.path>{child.label}</A>
                            </For>
                        </div>
                    })}

                    // Dark mode toggle — desktop only
                    {move || {
                        let is_dark = dark_mode_is_active.get();
                        let icon = if is_dark { BsMoon } else { BsSun };
                        view! {
                            <div class="hidden md:flex items-center gap-[5px]">
                                <ToggleSwitch
                                    initial_active_state=is_dark
                                    on:change=move |_| dark_mode_is_active.set(!dark_mode_is_active.get())
                                    id_attr="dark_mode_toggle"
                                />
                                <Icon icon=icon />
                            </div>
                        }
                    }}

                    // CTA — desktop, public only
                    {move || (!is_dashboard.get() && !is_blog.get()).then(|| view! {
                        <A
                            attr:class="hidden md:flex py-2 px-4 cursor-pointer rounded-[5px] border-2 border-primary text-primary hover:bg-primary hover:text-contrast-white font-bold"
                            href="/ratecard"
                        >
                            "Request Service"
                        </A>
                    })}

                    // Search icon — mobile, blog home only
                    {move || is_blog_home.get().then(|| view! {
                        <span
                            class="flex md:hidden items-center cursor-pointer"
                            on:click=move |_| ui_ctx.show_mobile_search.set(true)
                        >
                            <Icon width="16" height="16" icon=BsSearch />
                        </span>
                    })}

                    // Sign in — blog page, not authenticated
                    {move || {
                        let is_authenticated = auth_ctx.is_authenticated().get();
                        let on_blog_page = is_blog.get();
                        let is_dashboard = is_dashboard.get();

                        (on_blog_page && !is_authenticated && !is_dashboard).then(|| view! {
                            <A
                                attr:class="hidden md:flex py-2 px-4 cursor-pointer rounded-[5px] border-2 border-primary text-primary hover:bg-primary hover:text-contrast-white font-bold text-sm"
                                href="/sign-in"
                            >
                                "Sign In"
                            </A>
                        })
                    }}

                    // Profile pic + popover — dashboard or blog, has profile pic
                    {move || {
                        let profile_pic = user_profile.get().profile_picture;
                        let is_authenticated = auth_ctx.is_authenticated().get();
                        // let is_authenticated = is_authenticated.get();
                        ((is_dashboard.get() || is_blog.get()) && is_authenticated).then(|| {
                            profile_pic.map(|pic| view! {
                                <Popover
                                    showing=showing_user_popover
                                    display_item={
                                        let pic = pic.clone();
                                        move || view! {
                                            <img
                                                src=format!("{}?width=300", pic)
                                                class="size-[35px] object-cover rounded-full cursor-pointer"
                                                alt="dp"
                                            />
                                        }
                                    }
                                >
                                    <div class="flex flex-col gap-2 p-1">
                                        <A
                                            attr:class="py-2 px-4 hover:bg-primary hover:text-contrast-white cursor-pointer rounded-[5px] font-bold"
                                            href="/dashboard/user/profile"
                                        >
                                            <span class="flex items-center justify-between gap-4">
                                                <span>"Profile"</span>
                                                <Icon icon=MdiCardAccountDetailsOutline />
                                            </span>
                                        </A>
                                        <PermissionGuard
                                            match_mode=PermissionMatch::Any
                                            permissions=vec![
                                                "read:stats".into(),
                                                "write:portfolio".into(),
                                                "write:blog_post".into(),
                                                "write:media".into(),
                                            ]
                                        >
                                            <A
                                                attr:class="py-2 px-4 hover:bg-primary hover:text-contrast-white cursor-pointer rounded-[5px] font-bold"
                                                href="/dashboard"
                                            >
                                                <span class="flex items-center justify-between gap-4">
                                                    <span>"Dashboard"</span>
                                                    <Icon icon=MdiTabletDashboard />
                                                </span>
                                            </A>
                                        </PermissionGuard>
                                        <BasicButton
                                            style_ext="text-danger px-4 hover:bg-danger hover:text-contrast-white"
                                            onclick=handle_sign_out
                                        >
                                            <span class="flex items-center justify-between gap-4">
                                                <span>"Logout"</span>
                                                <Icon icon=BsPower />
                                            </span>
                                        </BasicButton>
                                    </div>
                                </Popover>
                            })
                        })
                    }}
                </div>
            </div>
        </div>
        </>
    }.into_any()
}
