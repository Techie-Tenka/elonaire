use detaxine_ui::components::{
    actions::button::{BasicButton, ButtonType},
    feedback::modal::modal::{BasicModal, UseCase},
};
use leptos::prelude::*;

use super::{sign_in_form::SignInForm, sign_up_form::SignUpForm};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMode {
    SignIn,
    SignUp,
}

impl Default for AuthMode {
    fn default() -> Self {
        Self::SignIn
    }
}

/// Tabbed sign-in / sign-up modal. Used by `ProtectedRoute` and anywhere
/// else that wants auth without leaving the current route.
///
/// `on_success` fires *after* the modal closes itself; callers can use it to
/// refresh data that was gated on auth.
#[component]
pub fn AuthModal(
    is_open: RwSignal<bool>,
    #[prop(optional)] initial_mode: AuthMode,
    #[prop(optional)] on_success: Option<Callback<()>>,
) -> impl IntoView {
    let mode = RwSignal::new(initial_mode);

    let switch_to_sign_up = Callback::new(move |_| mode.set(AuthMode::SignUp));
    let switch_to_sign_in = Callback::new(move |_| mode.set(AuthMode::SignIn));

    let handle_success = Callback::new(move |_| {
        is_open.set(false);
        if let Some(cb) = on_success {
            cb.run(());
        }
    });

    let title_signal = Signal::derive(move || match mode.get() {
        AuthMode::SignIn => "Sign In",
        AuthMode::SignUp => "Create Account",
    });

    view! {
        {
            move || {
                let title = title_signal.get().to_string();
                let sign_in_active = mode.get() == AuthMode::SignIn;

                // Shared tab base; active vs inactive adds the underline and
                // text color. `rounded-none` overrides BasicButton's default
                // radius so the underline sits flush against the border.
                let base = "px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px rounded-none";
                let active = format!("{base} border-primary text-primary");
                let inactive = format!("{base} border-transparent text-mid-gray hover:text-body");

                let sign_in_style = if sign_in_active { active.clone() } else { inactive.clone() };
                let sign_up_style = if sign_in_active { inactive } else { active };

                view! {
                    <BasicModal
                        title=title
                        is_open=is_open
                        use_case=UseCase::General
                        disable_auto_close=false
                        class="w-[70%] md:w-lg"
                        show_footer=false
                    >
                        <div class="flex flex-col items-center p-4">
                            // Tabs
                            <div class="flex gap-2 mb-6 border-b border-mid-gray w-full max-w-md">
                                <BasicButton
                                    button_text="Sign In"
                                    button_type=ButtonType::Button
                                    style_ext=sign_in_style.clone()
                                    on:click=move |_| mode.set(AuthMode::SignIn)
                                />
                                <BasicButton
                                    button_text="Sign Up"
                                    button_type=ButtonType::Button
                                    style_ext=sign_up_style.clone()
                                    on:click=move |_| mode.set(AuthMode::SignUp)
                                />
                            </div>

                            <Show
                                when=move || mode.get() == AuthMode::SignIn
                                fallback=move || view! {
                                    <SignUpForm
                                        on_success=handle_success
                                        on_switch_to_sign_in=switch_to_sign_in
                                    />
                                }
                            >
                                <SignInForm
                                    on_success=handle_success
                                    on_switch_to_sign_up=switch_to_sign_up
                                />
                            </Show>
                        </div>
                    </BasicModal>
                }
            }
        }
    }
}
