use detaxine_ui::components::feedback::modal::modal::{BasicModal, UseCase};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::data::context::{auth::use_auth, ui::use_ui};

#[component]
pub fn ErrorHandler(
    #[prop(optional, default = Callback::new(move |_| {}))] unauthorized_cb: Callback<()>,
) -> impl IntoView {
    let ui_ctx = use_ui();
    let auth_ctx = use_auth();
    let navigate = use_navigate();
    let error_modal_is_open = RwSignal::new(false);
    let (current_error, set_current_error) = signal(None::<String>);

    Effect::new(move |_| {
        if let Some(err) = ui_ctx.error.get() {
            if !auth_ctx.is_loading.get() {
                if err.is_unauthorized() {
                    ui_ctx.error.set(None);
                    unauthorized_cb.run(());
                } else if err.is_internal() {
                    ui_ctx.error.set(None);
                    navigate("/500", Default::default());
                } else {
                    set_current_error.set(Some(err.message().to_owned()));
                    error_modal_is_open.set(true);
                    ui_ctx.error.set(None);
                }
            }
        }
    });

    view! {
        <BasicModal title="Error" is_open=error_modal_is_open use_case=UseCase::Error disable_auto_close=false>
            <div class="p-[10px]">
                <p>{move || current_error.get()}</p>
            </div>
        </BasicModal>
    }.into_any()
}
