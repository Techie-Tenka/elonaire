use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::{
    data::{
        context::{auth::AuthContext, ui::UiContext},
        models::graphql::{
            email::{
                CreateSubscriptionResponse, CreateSubscriptionVars, SubscriberInput,
                SubscriptionInput, SubscriptionInputMetadata,
            },
            shared::{MessageInput, SendMessageResponse, SendMessageVars},
        },
    },
    utils::{errors::handle_graphql_errors, graphql_client::perform_mutation_or_query_with_vars},
};

const SHARED_SERVICE_API: Option<&str> = option_env!("SHARED_SERVICE_API");
const EMAIL_SERVICE_API: Option<&str> = option_env!("EMAIL_SERVICE_API");
const MARKETPLACE_WAITLIST_MAILING_LIST_ID: Option<&str> =
    option_env!("MARKETPLACE_WAITLIST_MAILING_LIST_ID");

const SEND_MESSAGE_QUERY: &str = r#"
    mutation SendMessage($message: MessageInput!) {
        sendMessage(message: $message) {
            data {
                subject
                body
                senderName
                senderEmail
                createdAt
                id
            }
            metadata {
                requestId
                newAccessToken
            }
        }
    }
"#;

const SUBSCRIBE_TO_MARKETPLACE_WAITLIST_QUERY: &str = r#"
    mutation SubscribeToMailingList($subscriptionInput: SubscriptionInput!) {
        subscribeToMailingList(subscriptionInput: $subscriptionInput) {
            data {
                createdAt
                id
                mailingList {
                    name
                    description
                    createdAt
                    id
                }
                subscriber {
                    email
                    firstName
                    lastName
                    status
                    createdAt
                    updatedAt
                    id
                }
            }
            metadata {
                requestId
                newAccessToken
            }
        }
    }
"#;

/// Utility context for one-off shared-service calls that don't warrant their
/// own context.
#[derive(Clone, Debug, Copy)]
pub struct SharedContext {
    ui: UiContext,
    auth: AuthContext,
    pub is_loading: RwSignal<bool>,
}

impl SharedContext {
    pub fn new(ui: UiContext, auth: AuthContext) -> Self {
        Self {
            ui,
            auth,
            is_loading: RwSignal::new(false),
        }
    }

    /// Fire-and-forget. `on_done(true)` on success, `on_done(false)` on any
    /// failure (which is also surfaced through `ui`). `is_loading` is
    /// managed by the context — the view reads it for its spinner.
    pub fn send_message(&self, message: MessageInput, on_done: Callback<bool>) {
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let Some(shared_service_api) = SHARED_SERVICE_API else {
                leptos::logging::error!("SHARED_SERVICE_API is not configured");
                ui.set_client_error("Messaging service is not configured.");
                loading.set(false);
                on_done.run(false);
                return;
            };

            let vars = SendMessageVars { message };
            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                SendMessageResponse,
                SendMessageVars,
            >(
                Some(&headers), shared_service_api, SEND_MESSAGE_QUERY, vars
            )
            .await;

            match response.get_data() {
                Some(_) => on_done.run(true),
                None => {
                    handle_graphql_errors(&response, &ui, None);
                    on_done.run(false);
                }
            }

            loading.set(false);
        });
    }

    /// Fire-and-forget. `on_done(true)` on success, `on_done(false)` on any
    /// failure (which is also surfaced through `ui`). `is_loading` is
    /// managed by the context — the view reads it for its spinner.
    pub fn subscribe_to_marketplace_waitlist(
        &self,
        subscriber: SubscriberInput,
        on_done: Callback<bool>,
    ) {
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let Some(mailing_list_id) = MARKETPLACE_WAITLIST_MAILING_LIST_ID else {
                leptos::logging::error!("MARKETPLACE_WAITLIST_MAILING_LIST_ID is not configured");
                ui.set_client_error("Waitlist is not configured.");
                loading.set(false);
                on_done.run(false);
                return;
            };

            let Some(email_service_api) = EMAIL_SERVICE_API else {
                leptos::logging::error!("EMAIL_SERVICE_API is not configured");
                ui.set_client_error("Waitlist is not configured.");
                loading.set(false);
                on_done.run(false);
                return;
            };

            let vars = CreateSubscriptionVars {
                subscription_input: SubscriptionInput {
                    subscriber,
                    subscription_input_metadata: SubscriptionInputMetadata {
                        mailing_list_id: mailing_list_id.into(),
                    },
                },
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateSubscriptionResponse,
                CreateSubscriptionVars,
            >(
                Some(&headers),
                email_service_api,
                SUBSCRIBE_TO_MARKETPLACE_WAITLIST_QUERY,
                vars,
            )
            .await;

            match response.get_data() {
                Some(_) => on_done.run(true),
                None => {
                    handle_graphql_errors(&response, &ui, None);
                    on_done.run(false);
                }
            }

            loading.set(false);
        });
    }
}

// ── Context helpers ─────────────────────────────────────────────────────

pub fn provide_shared(ui: UiContext, auth: AuthContext) -> SharedContext {
    let shared_ctx = SharedContext::new(ui, auth);
    provide_context(shared_ctx);
    shared_ctx
}

pub fn use_shared() -> SharedContext {
    expect_context::<SharedContext>()
}
