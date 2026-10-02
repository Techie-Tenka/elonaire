use leptos::prelude::*;
use leptos_meta::*;

use crate::{
    components::molecules::{headline::Headline, ratecard::RatecardComponent, top_nav::TopNav},
    data::context::billing::use_billing,
};

#[component]
pub fn Ratecard() -> impl IntoView {
    let billing_ctx = use_billing();
    let ratecards = move || billing_ctx.ratecards;

    Effect::new(move |_| {
        billing_ctx.fetch_ratecards();
    });

    view! {
        <Title text="My Ratecard"/>
        <main>
            <div class="min-h-svh flex flex-col gap-[40px]">
                <div class="sticky top-0 z-10 bg-contrast-white dark:bg-navy">
                    <TopNav />
                </div>
                <Headline title="My Ratecard" description="How much do I charge?" />
                <div class="display-constraints flex flex-col md:flex-row md:justify-center gap-[10px]">
                    <For
                        each=move || ratecards().get()
                        key=|ratecard| ratecard.id.clone()
                        children=move |ratecard| {
                            view! {
                                <RatecardComponent
                                    name=RwSignal::new(ratecard.name.as_ref().unwrap_or(&Default::default()).clone())
                                    services=RwSignal::new(ratecard.services.as_ref().unwrap_or(&Default::default()).to_vec())
                                />
                            }
                        }
                    />
                </div>
            </div>
        </main>
    }.into_any()
}
