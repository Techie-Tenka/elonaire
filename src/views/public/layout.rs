use detaxine_ui::components::{actions::button::BasicButton, forms::toggle_switch::ToggleSwitch};
use icondata::{
    BsInfoCircle, BsMoon, BsSun, IoClose, MdiCertificateOutline, MdiStore, MdiTrophyAward,
    RiArticleDocumentLine,
};
use leptos::{ev, prelude::*};
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::components::{A, Outlet};

use crate::{
    components::molecules::{footer::Footer, nav::Nav},
    data::context::ui::use_ui,
    views::{dashboard::layout::MenuItem, public::error_handler::ErrorHandler},
};

#[component]
pub fn MainLayout() -> impl IntoView {
    let ui_ctx = use_ui();
    let dark_mode_is_active = ui_ctx.dark_mode_is_active;

    // track collapsed state
    let (collapsed, set_collapsed) = signal(false);

    let handle_menu_click =
        move || Callback::new(move |_ev: ev::MouseEvent| set_collapsed.set(true));

    let menu_items = Memo::new(move |_| {
        vec![
            MenuItem::new("About", BsInfoCircle, "/about", vec![]),
            MenuItem::new("Resume", MdiCertificateOutline, "/resume", vec![]),
            MenuItem::new("Portfolio", MdiTrophyAward, "/portfolio", vec![]),
            MenuItem::new("Marketplace", MdiStore, "/marketplace", vec![]),
            MenuItem::new("Blog", RiArticleDocumentLine, "/blog", vec![]),
        ]
    });

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
        <Title text="Techie Tenka"/>
        <main>
            <ErrorHandler />
            <div class="relative min-h-svh">
                {/* Sidebar overlay */}
                <div
                    class=move || format!(
                        "fixed top-0 left-0 h-full transition-all duration-300 bg-contrast-white dark:bg-navy shadow-md z-40 {}",
                        if collapsed.get() { "w-64" } else { "w-0" }
                    )
                >
                    {/* Only render content if expanded */}
                    {move || if collapsed.get() {
                        Some(view! {
                            <div class="flex flex-col mx-[5%] min-h-svh">
                                <div class="flex items-center justify-between h-[47px] border-b border-light-gray">
                                    <p class="font-medium">NAVIGATION</p>
                                    <BasicButton
                                        style_ext="bg-transparent border-none"
                                        on:click=move |_| set_collapsed.set(false)
                                    >
                                        <Icon width="24" height="24" icon=IoClose />
                                    </BasicButton>
                                </div>
                                <nav class="flex flex-col">
                                    <For
                                        each=move || menu_items.get()
                                        key=|menu_item| menu_item.path.to_owned()
                                        let(child)
                                    >
                                        <div class="flex rounded-[5px] hover:bg-light-gray h-[45px]" on:click=move |_| set_collapsed.set(false)>
                                            <A attr:class="flex-1 h-full flex items-center gap-[10px]" href=child.path>
                                                <span><Icon width="24" height="24" icon=child.icon /></span>
                                                <span class="flex-1">{child.label}</span>
                                            </A>
                                        </div>
                                    </For>
                                </nav>
                                <div class="md:hidden flex flex-col gap-[10px]">
                                    <div class="flex items-center h-[40px] border-b border-light-gray">
                                        <p class="font-medium text-xs">ACTIONS</p>
                                    </div>
                                    <A attr:class="block md:hidden py-2 px-4 cursor-pointer rounded-[5px] border-2 border-primary text-primary hover:bg-primary hover:text-contrast-white font-bold text-center" href="/ratecard">"Request Service"</A>
                                </div>
                                <div class="md:hidden mt-auto flex flex-col gap-[10px]">
                                    <div class="flex items-center h-[40px] border-b border-light-gray">
                                        <p class="font-medium text-xs">PREFERENCES</p>
                                    </div>
                                    {
                                        move || {
                                            let is_dark = dark_mode_is_active.get();
                                            let icon = if !is_dark { BsSun } else { BsMoon };

                                            view! {
                                                <div class="flex md:hidden items-center gap-[5px]">
                                                    <ToggleSwitch
                                                    initial_active_state=is_dark
                                                    on:change=move |_| dark_mode_is_active.set(!dark_mode_is_active.get())
                                                    />
                                                    <Icon icon=icon />
                                                </div>
                                            }
                                        }
                                    }
                                </div>
                            </div>
                        })
                    } else {
                        None
                    }}
                </div>

                {/* Dark backdrop when sidebar is open */}
                {move || if collapsed.get() {
                    Some(view! {
                        <div
                            class="fixed inset-0 bg-contrast-white opacity-50 z-30"
                            on:click=move |_| set_collapsed.set(false)
                        />
                    })
                } else {
                    None
                }}

                {/* Main content */}
                <div class="min-h-svh flex flex-col gap-[40px]">
                    {/* Toggle button (opens sidebar) */}
                    <Nav onmenuclick=handle_menu_click() />
                    <Outlet />
                    <div class="mt-auto">
                        <Footer />
                    </div>
                </div>

            </div>
        </main>
    }.into_any()
}
