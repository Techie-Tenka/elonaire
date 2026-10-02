use detaxine_ui::utils::formatters::PipeOption;
use leptos::prelude::*;
use leptos_meta::*;

use crate::components::molecules::{
    flip_card::FlipCard, headline::Headline, section_title::SectionTitle, top_nav::TopNav,
};
use crate::data::context::{portfolio::use_portfolio, site_owner::use_site_owner};

#[component]
pub fn About() -> impl IntoView {
    let site_owner_ctx = use_site_owner();
    let portfolio_ctx = use_portfolio();

    Effect::new(move |_| {
        site_owner_ctx.fetch_site_owner_info();
        portfolio_ctx.fetch_services();
    });

    view! {
        <Title text="About"/>
        <main>
            <div class="min-h-svh flex flex-col gap-[40px]">
                <div class="sticky top-0 z-10 bg-contrast-white dark:bg-navy">
                    <TopNav />
                </div>
                <Headline title="About Me" description="Who Am I?" />
                <div class="flex flex-col md:flex-row md:justify-center md:gap-[20px] display-constraints">
                    <div class="max-w-[400px] h-[479px] relative md:basis-1/2">
                        <img src="https://techietenka.com/api/files/view/default/20230701_181652.jpg?width=600" alt="gallery-pic" class="rounded-[5px] w-[299px] h-[429px] object-cover"/>
                        <img src="https://techietenka.com/api/files/view/default/20240228_000643.jpg?width=600" alt="gallery-pic" class="rounded-[5px] w-[196px] h-[218px] absolute bottom-0 right-0 object-cover"/>
                    </div>
                    <div class="max-w-[400px] flex flex-col gap-[20px] md:basis-1/2">
                        <div class="flex flex-col gap-[20px]">
                            <h1>"Hello, I am "<span class="text-primary">{move || site_owner_ctx.site_owner_info.get().full_name}</span></h1>
                            <p>{move || site_owner_ctx.site_owner_info.get().bio}</p>
                        </div>

                        <div class="flex flex-col gap-[20px]">
                            <p><strong class="text-primary">"Age: "</strong>{move || site_owner_ctx.site_owner_info.get().age.text(None)}</p>
                            <p><strong class="text-primary">"Country of Residence: "</strong>Kenya</p>
                            <p><strong class="text-primary">"Relocation: "</strong>Open to relocation</p>
                        </div>
                    </div>
                </div>
                <div class="display-constraints">
                    <SectionTitle title="My Services" />
                </div>
                <div class="display-constraints grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-[20px]">
                    {
                        move || portfolio_ctx.services
                            .get()
                            .iter()
                            .map(|service| {
                                view! {
                                    <FlipCard title={service.title.clone().unwrap_or_default()} image_url={service.thumbnail.clone().unwrap_or_default()} description={service.description.clone().unwrap_or_default()} />
                                }
                            })
                            .collect::<Vec<_>>()
                    }
                </div>
            </div>
        </main>
    }.into_any()
}
