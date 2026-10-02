use leptos::prelude::*;

use crate::utils::errors::LocalErrorMessage;

#[derive(Clone, Debug, Copy)]
pub struct UiContext {
    pub error: RwSignal<Option<LocalErrorMessage>>,
    pub redirect_to: RwSignal<Option<String>>,
    pub show_mobile_search: RwSignal<bool>,
    pub dark_mode_is_active: RwSignal<bool>,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            error: RwSignal::new(None),
            redirect_to: RwSignal::new(None),
            show_mobile_search: RwSignal::new(false),
            dark_mode_is_active: RwSignal::new(false),
        }
    }

    pub fn set_error(&self, error: LocalErrorMessage, redirect_to: Option<&str>) {
        self.redirect_to
            .set(redirect_to.map(|link| link.to_string()));
        self.error.set(Some(error));
    }

    /// Convenience for client-side / UI-plumbing errors that have no
    /// server-issued code. Accepts anything `Into<LocalErrorMessage>` —
    /// in practice `&str` and `String`, both of which wrap into
    /// `LocalErrorMessage::Client`.
    ///
    /// Use [`set_error`] directly when you already have a `GraphQL` or
    /// `Rest` variant in hand, or when a redirect is needed.
    pub fn set_client_error(&self, msg: impl Into<LocalErrorMessage>) {
        self.set_error(msg.into(), None);
    }

    pub fn clear_error(&self) {
        self.error.set(None);
    }
}

// ── Context helpers ─────────────────────────────────────────────────────

pub fn provide_ui() -> UiContext {
    let ui_ctx = UiContext::new();
    provide_context(ui_ctx);
    ui_ctx
}

pub fn use_ui() -> UiContext {
    expect_context::<UiContext>()
}
