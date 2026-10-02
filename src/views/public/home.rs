use detaxine_ui::components::actions::button::BasicButton;
use icondata::{BsChevronDoubleDown, BsGithub};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::components::A;

use crate::{
    components::molecules::skeleton::Skeleton,
    data::context::{portfolio::use_portfolio, site_owner::use_site_owner},
};

#[component]
pub fn Home() -> impl IntoView {
    let portfolio_ctx = use_portfolio();
    let site_owner_ctx = use_site_owner();

    // State: id of the currently selected profession. Empty until the fetch lands.
    let (selected_profession, set_selected_profession) = signal(String::new());

    // Derived signal for the current description.
    let current_description = Memo::new(move |_| {
        portfolio_ctx
            .professions
            .get()
            .iter()
            .find(|r| r.id.clone().unwrap_or_default() == selected_profession.get())
            .map(|r| r.description.clone())
            .unwrap_or_default()
    });

    // Kick off both fetches on mount.
    Effect::new(move |_| {
        site_owner_ctx.fetch_site_owner_info();
        portfolio_ctx.fetch_professions();
    });

    // Pick a default selection once professions arrive.
    Effect::new(move |_| {
        let list = portfolio_ctx.professions.get();
        if selected_profession.get().is_empty() {
            if let Some(id) = list.first().and_then(|p| p.id.clone()) {
                set_selected_profession.set(id);
            }
        }
    });

    let ethics = vec![
        (
            "bg-[url('https://techietenka.com/api/files/view/default/vecteezy_a-businessman-works-on-his-laptop-at-home-with-a-virtual_19935805.jpg?width=800')]",
            "Commitment to Security",
            "From physical to application - I ensure security at every layer.",
            "md:row-span-1 md:col-span-1",
        ),
        (
            "bg-[url('https://techietenka.com/api/files/view/default/vecteezy_businessman-checking-documents-iso-standards-quality_12859725.jpg?width=800')]",
            "Commitment to Quality",
            "I never compromise on delivering exceptional results",
            "md:row-span-1 md:col-span-1",
        ),
        (
            "bg-[url('https://techietenka.com/api/files/view/default/medium-vecteezy_time-management-concept-businessman-manages-time-for_55787084_medium.jpg?width=800')]",
            "Timely Delivery",
            "Meeting deadlines is not negotiable",
            "md:row-span-1 md:col-span-2",
        ),
        (
            "bg-[url('https://techietenka.com/api/files/view/default/medium-vecteezy_man-hand-virtual-world-icon-communication-and-use-of-modern_4816140_medium.jpg?width=800')]",
            "Clear Communication",
            "Transparency at every step of the process",
            "md:row-span-2 md:col-span-2",
        ),
        (
            "bg-[url('https://techietenka.com/api/files/view/default/medium-vecteezy_concept-of-education-adds-new-skills-business-education_27547463_medium.jpg?width=800')]",
            "Continuous Learning",
            "Always staying ahead with latest technologies",
            "md:row-span-2 md:col-span-2",
        ),
        (
            "bg-[url('https://techietenka.com/api/files/view/default/medium-vecteezy_ai-generative-photo-american-business-male-people-shaking_29333390_medium.jpg?width=800')]",
            "Client-Focused",
            "Your success is my priority",
            "md:row-span-1 md:col-span-2",
        ),
    ];

    view! {
        <Title text="Techie Tenka"/>
        <div class="flex flex-col gap-[40px]">
            <div class="min-h-[90svh] flex flex-col gap-[40px] relative">
                // My User Info
                <div class="flex flex-col md:flex-row gap-[20px] md:justify-between md:items-center display-constraints">
                    <div class="flex flex-col gap-[20px] md:gap-[40px] text-center md:text-left">
                        <p class="text-salutation">Hello, I am</p>

                        // ── Name ─────────────────────────────────────────────
                        <p class="text-owner-name dark:text-light-gray min-h-[1em]">
                            <Show
                                when=move || !site_owner_ctx.is_loading.get()
                                fallback=move || view! {
                                    <span class="inline-flex flex-wrap items-center justify-center md:justify-start gap-3">
                                        <Skeleton class="h-[1em] w-[130px] bg-primary/40" />
                                        <Skeleton class="h-[1em] w-[220px]" />
                                    </span>
                                }
                            >
                                <span class="text-primary">
                                    {move || site_owner_ctx.site_owner_info.get().first_name}
                                </span>
                                {move || format!(
                                    " {} {}",
                                    site_owner_ctx.site_owner_info.get().middle_name.unwrap_or_default(),
                                    site_owner_ctx.site_owner_info.get().last_name.unwrap_or_default()
                                )}
                            </Show>
                        </p>

                        // ── Mobile profile picture ──────────────────────────
                        <Show
                            when=move || !site_owner_ctx.is_loading.get()
                            fallback=move || view! {
                                <Skeleton class="block h-[435px] w-full rounded-[5px] md:hidden" />
                            }
                        >
                            <img
                                alt="dp"
                                src={move || site_owner_ctx.site_owner_info.get().profile_picture}
                                class="h-[435px] object-cover rounded-[5px] md:hidden"
                            />
                        </Show>

                        // ── Professions ─────────────────────────────────────
                        <p class="text-2xl font-bold min-h-[2rem]">
                            <Show
                                when=move || !portfolio_ctx.is_loading.get()
                                fallback=move || view! {
                                    <span class="inline-flex items-center gap-3">
                                        <Skeleton class="h-[1em] w-[110px]" />
                                        <span class="opacity-40">"/"</span>
                                        <Skeleton class="h-[1em] w-[150px]" />
                                    </span>
                                }
                            >
                                {move || portfolio_ctx.professions.get().into_iter().enumerate().map(|(idx, profession)| {
                                    let occupation = profession.occupation.unwrap_or_default();
                                    let profession_id = profession.id.unwrap_or_default();
                                    let is_selected = {
                                        let profession_id = profession_id.clone();
                                        move || selected_profession.get() == profession_id
                                    };
                                    let profession_id_for_click = profession_id.clone();
                                    let on_click = move |_| {
                                        set_selected_profession.set(profession_id_for_click.clone())
                                    };

                                    let is_last = move || idx == portfolio_ctx.professions.get().len() - 1;

                                    view! {
                                        <span
                                            class=move || format!("font-bold {}", if is_selected() {
                                                "text-primary underline cursor-default"
                                            } else {
                                                "cursor-pointer hover:text-primary"
                                            })
                                            on:click=on_click
                                        >
                                            {occupation}
                                        </span>
                                        {move || if is_last() { "" } else { " / " }}
                                    }
                                }).collect::<Vec<_>>()}
                            </Show>
                        </p>

                        // ── Description ─────────────────────────────────────
                        <p class="min-h-[90px] max-w-[600px] text-base">
                            <Show
                                when=move || !portfolio_ctx.is_loading.get()
                                fallback=move || view! {
                                    <span class="flex flex-col gap-2 pt-1">
                                        <Skeleton class="block h-3.5 w-full" />
                                        <Skeleton class="block h-3.5 w-[92%]" />
                                        <Skeleton class="block h-3.5 w-[65%]" />
                                    </span>
                                }
                            >
                                {current_description}
                            </Show>
                        </p>

                        // ── GitHub button ───────────────────────────────────
                        <Show
                            when=move || !portfolio_ctx.is_loading.get()
                            fallback=move || view! {
                                <Skeleton class="block h-[44px] w-full rounded-[5px] md:w-[292px]" />
                            }
                        >
                            <A attr:class="font-bold py-2 px-4 cursor-pointer rounded-[5px] bg-primary text-contrast-white md:w-[292px] flex items-center justify-center gap-2"
                               href={move || site_owner_ctx.site_owner_info.get()
                                   .socials
                                   .as_ref()
                                   .and_then(|socials| socials.iter().find(|s| s.name.to_lowercase() == "github"))
                                   .and_then(|social| Some(social.url.clone())).unwrap_or_default()}
                               target="_blank">
                                <span>"Checkout my GitHub"</span>
                                <span><Icon width="24" height="24" icon=BsGithub /></span>
                            </A>
                        </Show>
                    </div>

                    // ── Desktop blob image ──────────────────────────────────
                    <div class="hidden md:block w-[50%]">
                        <svg width="0" height="0" class="absolute">
                            <defs>
                                <clipPath id="blob-clip" clipPathUnits="objectBoundingBox">
                                    <path d="
                                        M 0.15,0.05
                                        C 0.3,-0.08 0.55,-0.02 0.7,0.04
                                        C 0.85,0.1 1.02,0.18 0.98,0.35
                                        C 0.94,0.5 1.06,0.62 0.97,0.75
                                        C 0.88,0.88 0.72,1.04 0.55,0.99
                                        C 0.38,0.94 0.25,1.06 0.12,0.97
                                        C -0.02,0.88 -0.06,0.72 0.03,0.58
                                        C 0.1,0.45 -0.04,0.32 0.04,0.2
                                        C 0.08,0.12 0.05,0.14 0.15,0.05
                                        Z
                                    " />
                                </clipPath>
                            </defs>
                        </svg>

                        <Show
                            when=move || !portfolio_ctx.is_loading.get()
                            fallback=move || view! {
                                <Skeleton class="block aspect-square w-full rounded-[5px]" />
                            }
                        >
                            <div class="bg-contrast-white dark:bg-transparent rounded-[5px] aspect-square">
                                <img
                                    alt="dp"
                                    src={move || site_owner_ctx.site_owner_info.get().profile_picture}
                                    class="object-cover w-full h-full mix-blend-multiply dark:mix-blend-normal"
                                    style="clip-path: url(#blob-clip);"
                                />
                            </div>
                        </Show>
                    </div>
                </div>

                {/* Scroll button */}
                <BasicButton style_ext="hidden absolute bottom-8 left-1/2 -translate-x-1/2 md:flex flex-col items-center gap-2 text-sm opacity-60 hover:opacity-100 transition-opacity duration-300 cursor-pointer animate-bounce text-secondary">
                    <span class="text-xs tracking-widest uppercase">"Scroll"</span>
                    <Icon icon=BsChevronDoubleDown width="2rem" height="2rem" />
                </BasicButton>
            </div>

            // My work ethic
            <section class="flex flex-col justify-center">
                <div class="grid grid-cols-1 md:grid-cols-4 md:grid-rows-4 gap-[10px] md:auto-rows-[200px] display-constraints">

                        <div class="md:row-span-1 md:col-span-2 bg-primary text-contrast-white rounded-[5px] p-8 flex flex-col justify-center min-h-[200px]">
                            <h2 class="text-3xl md:text-5xl font-bold mb-4 text-contrast-white">"My Work Ethic"</h2>
                            <p class="text-lg opacity-90">"The principles that drive my work"</p>
                        </div>

                        <For
                            each=move || ethics.clone()
                            key=|(bg, title, _, _)| format!("{}{}", bg, title)
                            children=move |(bg_image, title, description, span_class)| {
                                view! {
                                    <div class=format!("row-span-1 {} relative rounded-[5px] overflow-hidden group cursor-pointer min-h-[200px]", span_class)>
                                        <div
                                            class=format!("absolute inset-0 bg-cover bg-center transition-transform duration-500 group-hover:scale-110 {}", bg_image)
                                        ></div>
                                        <div class="absolute inset-0 bg-black/40 group-hover:bg-black/60 transition-colors duration-300"></div>
                                        <div class="relative h-full p-4 md:p-6 flex flex-col justify-end text-contrast-white">
                                            <h3 class="text-lg md:text-xl font-bold mb-2 text-contrast-white">{title}</h3>
                                            <p class="text-xs md:text-sm opacity-0 group-hover:opacity-100 transition-opacity duration-300">{description}</p>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
            </section>

            <div class="flex flex-col bg-primary">
                <div class="flex-1 relative overflow-hidden h-[300px] md:h-[400px] display-constraints">
                    <div
                        class="absolute inset-0 bg-cover bg-center rounded-[5px]"
                        style="background-image: url('https://techietenka.com/api/files/view/default/medium-vecteezy_a-man-and-virtual-icon-modern-cyberspace-network-technology_24162388_medium.jpg?width=1500');"
                    />

                    <div
                        class="absolute inset-0 bg-primary"
                        style="clip-path: polygon(0 0, 60% 0, 40% 100%, 0 100%);"
                    />

                    <div class="relative z-10 flex flex-col gap-[40px] items-center justify-center h-full p-[10%]">
                        <p class="text-owner-name text-contrast-white text-center">
                            "Let's turn that technical idea you have in mind into reality. Shall we?"
                        </p>
                        <A
                            attr:class="py-2 px-4 cursor-pointer rounded-[5px] border-2 border-contrast-white text-contrast-white hover:bg-contrast-white hover:text-primary font-bold"
                            href="/ratecard"
                        >
                            "Request Service"
                        </A>
                    </div>
                </div>
            </div>
        </div>
    }.into_any()
}
