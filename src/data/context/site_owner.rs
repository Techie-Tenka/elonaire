use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::{
    data::{
        context::{auth::AuthContext, ui::UiContext},
        models::graphql::acl::{FetchSiteOwnerResponse, User},
    },
    utils::{errors::handle_graphql_errors, graphql_client::perform_query_without_vars},
};

const ACL_SERVICE_API: Option<&str> = option_env!("ACL_SERVICE_API");

#[derive(Clone, Debug, Copy)]
pub struct SiteOwnerContext {
    ui: UiContext,
    auth: AuthContext,
    pub site_owner_info: RwSignal<User>,
    pub is_loading: RwSignal<bool>,
}

impl SiteOwnerContext {
    pub fn new(ui: UiContext, auth: AuthContext) -> Self {
        Self {
            ui,
            auth,
            site_owner_info: RwSignal::new(User::default()),
            is_loading: RwSignal::new(false),
        }
    }

    pub fn fetch_site_owner_info(&self) {
        let site_owner_sig = self.site_owner_info;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_site_owner_query = r#"
                   query FetchSiteOwnerInfo {
                        fetchSiteOwnerInfo {
                            data {
                                firstName
                                middleName
                                lastName
                                gender
                                dob
                                email
                                country
                                createdAt
                                updatedAt
                                profilePicture
                                bio
                                website
                                address
                                id
                                fullName
                                age
                                socials {
                                    name
                                    url
                                }
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchSiteOwnerResponse>(
                Some(&headers),
                acl_service_api,
                fetch_site_owner_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_site_owner_info
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_owned();
                    site_owner_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }
}

// ── Context helpers ─────────────────────────────────────────────────────

pub fn provide_site_owner(ui: UiContext, auth: AuthContext) -> SiteOwnerContext {
    let site_owner_ctx = SiteOwnerContext::new(ui, auth);
    provide_context(site_owner_ctx);
    site_owner_ctx
}

pub fn use_site_owner() -> SiteOwnerContext {
    expect_context::<SiteOwnerContext>()
}
