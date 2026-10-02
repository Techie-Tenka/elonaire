use detaxine_ui::{
    components::{
        content::collapse::{Collapse, PanelInfo},
        data_display::timeline::{Timeline, TimelineItem, TimelineStatus},
    },
    utils::{formatters::PipeOption, time::convert_date_to_human_readable_format},
};
use leptos::prelude::*;
use leptos_meta::*;

use crate::{
    components::molecules::{headline::Headline, section_title::SectionTitle, top_nav::TopNav},
    data::{
        context::portfolio::use_portfolio,
        models::graphql::shared::{
            UserResume, UserResumeSection, UserSkill, UserSkillLevel, UserSkillType,
        },
    },
};

#[component]
pub fn Resume() -> impl IntoView {
    let portfolio_ctx = use_portfolio();
    let resume = move || portfolio_ctx.resume;
    let skills = move || portfolio_ctx.skills;

    Effect::new(move |_| {
        portfolio_ctx.fetch_resume();
        portfolio_ctx.fetch_skills();
    });

    view! {
        <Title text="Resume"/>
        <main>
            <div class="min-h-svh flex flex-col gap-[40px]">
                <div class="sticky top-0 z-10 bg-contrast-white dark:bg-navy">
                    <TopNav />
                </div>
                <Headline title="Resume" description="I am available for work" />
                <div class="display-constraints flex flex-col md:flex-row gap-[40px]">
                    <div class="w-full md:basis-1/2 flex flex-col gap-[10px]">
                        <SectionTitle title="Education" />
                        {
                            move || {
                                let education_timeline_items = RwSignal::new(resume()
                                    .get()
                                    .iter()
                                    .filter(|resume| resume.section.as_ref() == Some(&UserResumeSection::Education))
                                    .map(generate_timeline_item)
                                    .collect::<Vec<TimelineItem>>());

                                view! {
                                    <Timeline steps=education_timeline_items />
                                }
                            }
                        }
                    </div>
                    <div class="w-full md:basis-1/2 flex flex-col gap-[10px]">
                        <SectionTitle title="Work Experience" />
                        {
                            move || {
                                let experience_timeline_items = RwSignal::new(resume()
                                    .get()
                                    .iter()
                                    .filter(|resume| resume.section.as_ref() == Some(&UserResumeSection::Experience))
                                    .map(generate_timeline_item)
                                    .collect::<Vec<TimelineItem>>());

                                view! {
                                    <Timeline steps=experience_timeline_items />
                                }
                            }
                        }
                    </div>
                </div>
                <div class="display-constraints flex flex-col md:flex-row gap-[40px]">
                    <div class="w-full md:basis-1/2 flex flex-col gap-[10px]">
                        <SectionTitle title="Technical Skills" />
                        {
                            move || {
                                let technical_skills = RwSignal::new(skills()
                                    .get()
                                    .iter()
                                    .filter(|skill| skill.skill_type.as_ref() == Some(&UserSkillType::Technical))
                                    .map(generate_panel_info)
                                    .collect::<Vec<PanelInfo>>());

                                view! {
                                    <Collapse is_accordion=true panel_items=technical_skills />
                                }
                            }
                        }
                    </div>
                    <div class="w-full md:basis-1/2 flex flex-col gap-[10px]">
                        <SectionTitle title="Soft Skills" />
                        {
                            move || {
                                let soft_skills = RwSignal::new(skills()
                                    .get()
                                    .iter()
                                    .filter(|skill| skill.skill_type.as_ref() == Some(&UserSkillType::Soft))
                                    .map(generate_panel_info)
                                    .collect::<Vec<PanelInfo>>());

                                view! {
                                    <Collapse is_accordion=true panel_items=soft_skills />
                                }
                            }
                        }
                    </div>
                </div>
            </div>
        </main>
    }
    .into_any()
}

/// Utility function to generate resume timeline items.
fn generate_timeline_item(resume: &UserResume) -> TimelineItem {
    let start_date = convert_date_to_human_readable_format(
        resume.start_date.as_ref().unwrap_or(&Default::default()),
    );
    let end_date = match resume.end_date.as_ref() {
        Some(date) => convert_date_to_human_readable_format(date),
        None => "Present".into(),
    };
    let years_of_experience = match resume.years_of_experience.as_ref() {
        Some(years) => match years {
            0 => "< 1 year".to_string(),
            1 => "1 year".to_string(),
            _ => format!("{} years", years),
        },
        None => "".to_string(),
    };

    let achievements = resume.achievements.as_ref().unwrap_or(&vec![]).to_vec();

    TimelineItem {
        time_info: format!("{start_date} - {end_date} ({years_of_experience})"),
        title: resume
            .title
            .as_ref()
            .unwrap_or(&Default::default())
            .to_owned(),
        more_info: Some(
            resume
                .more_info
                .as_ref()
                .unwrap_or(&String::new())
                .to_owned(),
        ),
        status: TimelineStatus::Neutral,
        content: ViewFn::from(move || {
            view! {
                <ul class="list-disc list-inside">
                    {
                        achievements.iter().map(|achievement| {
                            view! { <li>{achievement.description.clone()}</li> }
                        }).collect::<Vec<_>>()
                    }
                </ul>
            }
        }),
        ..Default::default()
    }
}

/// A utility function to generate a `UserSkill` `PanelInfo`.
fn generate_panel_info(skill: &UserSkill) -> PanelInfo {
    let skill = skill.clone();

    let skill_ref = &skill;
    let title_skill = skill_ref.clone();
    let children_skill = skill_ref.clone();

    PanelInfo {
        id: skill_ref.id.as_ref().cloned().unwrap_or_default(),
        title: ViewFn::from(move || {
            let level = title_skill
                .level
                .as_ref()
                .unwrap_or(&UserSkillLevel::Beginner)
                .clone();

            view! {
                <div class="flex-1 flex flex-row items-center justify-between">
                    <img src={title_skill.thumbnail.text(None)} alt="skill-img" class="size-7 rounded-[5px] object-cover" />
                    <p class="font-bold">{title_skill.name.text(None)}</p>
                    <p class="text-xs">{format!("{:?}", level)}</p>
                </div>
            }
        }),
        children: ViewFn::from(move || {
            view! {
                <p>{children_skill.description.text(None)}</p>
            }
        }),
        ..Default::default()
    }
}
