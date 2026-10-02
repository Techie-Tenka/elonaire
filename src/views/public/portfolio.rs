use detaxine_ui::components::navigation::tabs::{Tab, TabLabel, Tabs};
use leptos::prelude::*;
use leptos_meta::*;

use crate::components::molecules::flip_card::FlipCard;
use crate::components::molecules::{headline::Headline, top_nav::TopNav};
use crate::data::context::portfolio::use_portfolio;
use crate::data::models::graphql::shared::{UserPortfolio, UserPortfolioCategory};
use crate::utils::custom_traits::EnumerableEnum;

#[component]
pub fn Portfolio() -> impl IntoView {
    let portfolio_ctx = use_portfolio();
    let portfolio = move || portfolio_ctx.portfolio;

    let javascript_projects = RwSignal::new(vec![] as Vec<UserPortfolio>);
    let rust_projects = RwSignal::new(vec![] as Vec<UserPortfolio>);
    let database_projects = RwSignal::new(vec![] as Vec<UserPortfolio>);
    let devops_projects = RwSignal::new(vec![] as Vec<UserPortfolio>);
    let cloud_projects = RwSignal::new(vec![] as Vec<UserPortfolio>);
    let mobile_projects = RwSignal::new(vec![] as Vec<UserPortfolio>);

    let portfolio_tabs = RwSignal::new(
        UserPortfolioCategory::variants_slice()
            .iter()
            .map(|category| {
                let owned_category = category.to_string();
                TabLabel::new(ViewFn::from(move || {
                    let owned_category = owned_category.clone();
                    view! { <p>{owned_category}</p> }
                }))
            })
            .collect::<Vec<TabLabel>>(),
    );

    // Filter once into per-category signals.
    Effect::new(move || {
        javascript_projects.set(
            portfolio()
                .get()
                .iter()
                .filter(|project| {
                    project.category.as_ref() == Some(&UserPortfolioCategory::JavaScript)
                })
                .cloned()
                .collect(),
        );
        rust_projects.set(
            portfolio()
                .get()
                .iter()
                .filter(|project| project.category.as_ref() == Some(&UserPortfolioCategory::Rust))
                .cloned()
                .collect(),
        );
        database_projects.set(
            portfolio()
                .get()
                .iter()
                .filter(|project| {
                    project.category.as_ref() == Some(&UserPortfolioCategory::Database)
                })
                .cloned()
                .collect(),
        );
        devops_projects.set(
            portfolio()
                .get()
                .iter()
                .filter(|project| project.category.as_ref() == Some(&UserPortfolioCategory::DevOps))
                .cloned()
                .collect(),
        );
        cloud_projects.set(
            portfolio()
                .get()
                .iter()
                .filter(|project| project.category.as_ref() == Some(&UserPortfolioCategory::Cloud))
                .cloned()
                .collect(),
        );
        mobile_projects.set(
            portfolio()
                .get()
                .iter()
                .filter(|project| project.category.as_ref() == Some(&UserPortfolioCategory::Mobile))
                .cloned()
                .collect(),
        );
    });

    // Fetch on mount.
    Effect::new(move |_| {
        portfolio_ctx.fetch_portfolio();
    });

    view! {
        <Title text="Portfolio"/>
        <main>
            <div class="min-h-svh flex flex-col gap-[40px]">
                <div class="sticky top-0 z-10 bg-contrast-white dark:bg-navy">
                    <TopNav />
                </div>
                <Headline title="Portfolio" description="Showcase of my best work" />
                <div class="display-constraints flex flex-col md:flex-row gap-[40px]">
                    <Tabs tab_labels=portfolio_tabs>
                        <Tab slot>
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-[20px]">
                                {
                                    move || javascript_projects.get().iter().map(|project| {
                                        view! {
                                            <FlipCard title={project.title.as_ref().unwrap_or(&Default::default()).clone()} image_url={project.thumbnail.as_ref().unwrap_or(&Default::default()).clone()} description={project.description.as_ref().unwrap_or(&Default::default()).clone()} />
                                        }
                                    }).collect::<Vec<_>>()
                                }
                            </div>
                        </Tab>
                        <Tab slot>
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-[20px]">
                                {
                                    move || rust_projects.get().iter().map(|project| {
                                        view! {
                                            <FlipCard title={project.title.as_ref().unwrap_or(&Default::default()).clone()} image_url={project.thumbnail.as_ref().unwrap_or(&Default::default()).clone()} description={project.description.as_ref().unwrap_or(&Default::default()).clone()} />
                                        }
                                    }).collect::<Vec<_>>()
                                }
                            </div>
                        </Tab>
                        <Tab slot>
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-[20px]">
                                {
                                    move || database_projects.get().iter().map(|project| {
                                        view! {
                                            <FlipCard title={project.title.as_ref().unwrap_or(&Default::default()).clone()} image_url={project.thumbnail.as_ref().unwrap_or(&Default::default()).clone()} description={project.description.as_ref().unwrap_or(&Default::default()).clone()} />
                                        }
                                    }).collect::<Vec<_>>()
                                }
                            </div>
                        </Tab>
                        <Tab slot>
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-[20px]">
                                {
                                    move || devops_projects.get().iter().map(|project| {
                                        view! {
                                            <FlipCard title={project.title.as_ref().unwrap_or(&Default::default()).clone()} image_url={project.thumbnail.as_ref().unwrap_or(&Default::default()).clone()} description={project.description.as_ref().unwrap_or(&Default::default()).clone()} />
                                        }
                                    }).collect::<Vec<_>>()
                                }
                            </div>
                        </Tab>
                        <Tab slot>
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-[20px]">
                                {
                                    move || cloud_projects.get().iter().map(|project| {
                                        view! {
                                            <FlipCard title={project.title.as_ref().unwrap_or(&Default::default()).clone()} image_url={project.thumbnail.as_ref().unwrap_or(&Default::default()).clone()} description={project.description.as_ref().unwrap_or(&Default::default()).clone()} />
                                        }
                                    }).collect::<Vec<_>>()
                                }
                            </div>
                        </Tab>
                        <Tab slot>
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-[20px]">
                                {
                                    move || mobile_projects.get().iter().map(|project| {
                                        view! {
                                            <FlipCard title={project.title.as_ref().unwrap_or(&Default::default()).clone()} image_url={project.thumbnail.as_ref().unwrap_or(&Default::default()).clone()} description={project.description.as_ref().unwrap_or(&Default::default()).clone()} />
                                        }
                                    }).collect::<Vec<_>>()
                                }
                            </div>
                        </Tab>
                    </Tabs>
                </div>
            </div>
        </main>
    }.into_any()
}
