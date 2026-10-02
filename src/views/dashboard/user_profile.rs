use detaxine_ui::{
    components::actions::button::BasicButton,
    utils::formatters::{Pipe, PipeOption},
};
use icondata::{BsCamera, BsPencil};
use leptos::prelude::*;
use leptos_icons::Icon;

use crate::data::context::{auth::use_auth, user::use_user};

#[component]
pub fn ProfilePage() -> impl IntoView {
    let user_ctx = use_user();
    let auth_ctx = use_auth();

    view! {
        <div class="min-h-svh flex flex-col gap-[40px] display-constraints">
            {
                move || {
                    let user = user_ctx.user_profile.get();
                    let current_role = auth_ctx.current_role.get();
                    view! {
                        // Avatar card
                        <div class="bg-white dark:bg-navy-light rounded-[5px] shadow-sm p-6 mb-4 flex items-center gap-5">
                            <div class="relative w-20 h-20 shrink-0">
                                <img
                                    src=format!("{}?width=300", user.profile_picture.unwrap_or_default())
                                    class="w-20 h-20 rounded-full object-cover border-4 border-white shadow"
                                />
                                <BasicButton style_ext="absolute bottom-0 right-0 bg-primary text-white rounded-full p-1.5! shadow flex items-center justify-center">
                                    <Icon icon=BsCamera />
                                </BasicButton>
                            </div>
                            <div>
                                <h2 class="text-lg font-semibold">{user.full_name.text(None)}</h2>
                                <p class="text-sm">{current_role.text(None)}</p>
                                <p class="text-sm">"Leeds, United Kingdom"</p>
                            </div>
                        </div>

                        // Personal Information card
                        <div class="bg-white dark:bg-navy-light rounded-[5px] shadow-sm p-6 mb-4">
                            <div class="flex items-center justify-between mb-4">
                                <h3 class="text-base font-semibold">"Personal Information"</h3>
                                <BasicButton style_ext="bg-primary text-white text-sm font-medium px-4 py-2 rounded-[5px] hover:bg-secondary transition-colors">
                                    <Icon icon=BsPencil />
                                </BasicButton>
                            </div>
                            <div class="border-t border-light-gray pt-4 grid grid-cols-1 sm:grid-cols-3 gap-6">
                                <div>
                                    <p class="text-xs mb-1">"First Name"</p>
                                    <p class="text-sm font-semibold">{user.first_name.text(None)}</p>
                                </div>
                                <div>
                                    <p class="text-xs mb-1">"Last Name"</p>
                                    <p class="text-sm font-semibold">{user.last_name.text(None)}</p>
                                </div>
                                <div>
                                    <p class="text-xs mb-1">"Date of Birth"</p>
                                    <p class="text-sm font-semibold">{user.dob.text(None)}</p>
                                </div>
                                <div>
                                    <p class="text-xs mb-1">"Email Address"</p>
                                    <p class="text-sm font-semibold">{user.email.text(None)}</p>
                                </div>
                                <div>
                                    <p class="text-xs mb-1">"Phone Number"</p>
                                    <p class="text-sm font-semibold">{user.phone.text(None)}</p>
                                </div>
                                <div>
                                    <p class="text-xs mb-1">"User Role"</p>
                                    <p class="text-sm font-semibold">"Guest"</p>
                                </div>
                            </div>
                        </div>

                        // Address card
                        <div class="bg-white dark:bg-navy-light rounded-[5px] shadow-sm p-6">
                            <div class="flex items-center justify-between mb-4">
                                <h3 class="text-base font-semibold">"Address"</h3>
                                <BasicButton style_ext="border border-light-gray text-sm font-medium px-4 py-2 rounded-[5px] hover:bg-gray/20 transition-colors">
                                    <Icon icon=BsPencil />
                                </BasicButton>
                            </div>
                            <div class="border-t border-light-gray pt-4 grid grid-cols-1 sm:grid-cols-3 gap-6">
                                <div>
                                    <p class="text-xs mb-1">"Country"</p>
                                    <p class="text-sm font-semibold">{user.country.text(None)}</p>
                                </div>
                                <div>
                                    <p class="text-xs mb-1">"City"</p>
                                    <p class="text-sm font-semibold">"Leeds, East London"</p>
                                </div>
                                <div>
                                    <p class="text-xs mb-1">"Postal Code"</p>
                                    <p class="text-sm font-semibold">"ERT 1254"</p>
                                </div>
                            </div>
                        </div>
                    }
                }
            }
        </div>
    }.into_any()
}
