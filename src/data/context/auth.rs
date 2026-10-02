use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::{
    data::{
        context::ui::UiContext,
        models::{
            general::{
                acl::{AuthCode, AuthDetails, OauthClientName},
                shared::RestResponse,
            },
            graphql::acl::{
                AuthStatus, CheckAuthResponse, SignInResponse, SignInVars, SignOutResponse,
                UserLoginsInput,
            },
        },
    },
    utils::{
        errors::handle_graphql_errors,
        graphql_client::{perform_mutation_or_query_with_vars, perform_query_without_vars},
    },
};

const ACL_SERVICE_API: Option<&str> = option_env!("ACL_SERVICE_API");

#[derive(Clone, Debug, Copy)]
pub struct AuthContext {
    ui: UiContext,
    pub token: RwSignal<Option<String>>,
    pub current_role: RwSignal<String>,
    pub current_role_permissions: RwSignal<Vec<String>>,
    pub is_loading: RwSignal<bool>,
    pub oauth_redirect_url: RwSignal<Option<String>>,
    pub signed_out_dirty: RwSignal<u64>,
    pub user_id: RwSignal<Option<String>>,
}

impl AuthContext {
    pub fn new(ui: UiContext) -> Self {
        Self {
            ui,
            token: RwSignal::new(None),
            current_role: RwSignal::new(String::new()),
            current_role_permissions: RwSignal::new(Vec::new()),
            is_loading: RwSignal::new(false),
            oauth_redirect_url: RwSignal::new(None),
            signed_out_dirty: RwSignal::new(0),
            user_id: RwSignal::new(None),
        }
    }

    /// Derived, not stored — mirrors the old `!token.is_empty()` check in SignIn.rs.
    /// Callers watch this (and/or `is_loading`) reactively instead of awaiting.
    pub fn is_authenticated(&self) -> Signal<bool> {
        let token = self.token;
        Signal::derive(move || token.get().is_some())
    }

    /// Verifies the token currently held in `self.token` against the server
    /// via `checkAuth`, then hydrates `user_id`/`current_role`/
    /// `current_role_permissions`.
    ///
    /// The token is the single source of truth — callers are responsible for
    /// putting it into the context *before* calling this (sign-in response,
    /// OAuth callback, or a persisted token on app boot). If no token is
    /// present, this is a no-op.
    ///
    /// On any failure (missing API config, transport error, GraphQL error,
    /// or `isAuth: false`), `clear()` wipes all session state — a stale
    /// role/permission set can never survive a failed re-auth.
    ///
    /// Callers react to `is_authenticated()`/`is_loading` via Effect rather
    /// than awaiting this.
    pub fn authenticate_with_token(&self) {
        let this = *self;
        let user_id_sig = self.user_id;
        let current_role_sig = self.current_role;
        let permissions_sig = self.current_role_permissions;
        let loading = self.is_loading;
        let ui = self.ui;

        loading.set(true);

        spawn_local(async move {
            let check_auth_query = r#"
                query CheckAuth {
                    checkAuth {
                        data {
                            isAuth
                            sub
                            currentRole
                            newAccessToken
                            currentRolePermissions
                        }
                        metadata {
                            requestId
                            newAccessToken
                        }
                    }
                }
               "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                this.clear();
                loading.set(false);
                return;
            };

            // Headers are derived from the token already in the context.
            let headers = this.headers();

            let response = perform_query_without_vars::<CheckAuthResponse>(
                Some(&headers),
                acl_service_api,
                check_auth_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let auth_status: AuthStatus = data
                        .check_auth
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data();

                    // Server answered cleanly but rejected the token.
                    if !auth_status.is_auth {
                        this.clear();
                        loading.set(false);
                        return;
                    }

                    user_id_sig.set(Some(auth_status.sub));
                    current_role_sig.set(auth_status.current_role);
                    permissions_sig.set(auth_status.current_role_permissions);

                    // Honor rotation. Prefer the `data` field; the metadata
                    // copy is a fallback for servers that only populate there.
                    if let Some(rotated) = auth_status.new_access_token {
                        this.token.set(Some(rotated));
                    }
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                    this.clear();
                }
            }

            loading.set(false);
        });
    }

    pub fn sign_out(&self) {
        let this = *self;
        let loading = self.is_loading;
        let dirty = self.signed_out_dirty;
        let ui = self.ui;

        loading.set(true);

        spawn_local(async move {
            let query = r#"
                    mutation SignOut {
                        signOut {
                            data
                            metadata {
                                requestId
                                newAccessToken
                            }
                        }
                    }
                   "#;

            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = this.headers();

            let response = perform_query_without_vars::<SignOutResponse>(
                Some(&headers),
                acl_service_api,
                query,
            )
            .await;

            match response.get_data() {
                Some(_data) => {
                    this.clear();
                    dirty.update(|n| *n += 1);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    /// Fire-and-forget. Requests a provider redirect URL for social sign-in
    /// and stores it in `oauth_redirect_url`; the view is responsible for
    /// actually navigating there (browser `window.open`, not app routing).
    pub fn sign_in_with_oauth(&self, client: OauthClientName) {
        let redirect_url_sig = self.oauth_redirect_url;
        let loading = self.is_loading;
        let ui = self.ui;

        loading.set(true);

        spawn_local(async move {
            let user_logins = SignInVars {
                raw_user_details: UserLoginsInput {
                    user_name: None,
                    password: None,
                    oauth_client: Some(client),
                },
            };

            let query = r#"
                       mutation SignIn($rawUserDetails: UserLoginsInput!) {
                           signIn(rawUserDetails: $rawUserDetails) {
                               data {
                                   url
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

            let response = perform_mutation_or_query_with_vars::<SignInResponse, SignInVars>(
                None,
                acl_service_api,
                query,
                user_logins,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    if let Some(auth_details) = &data.sign_in {
                        let url = auth_details
                            .get_data()
                            .url
                            .unwrap_or_else(|| "/sign-in".into());
                        redirect_url_sig.set(Some(url));
                    }
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    /// Fire-and-forget. Signs in with username/password, stores the issued
    /// token, then delegates to `authenticate_with_token` to verify it.
    pub fn sign_in_with_credentials(&self, user_logins: UserLoginsInput) {
        let this = *self;
        let loading = self.is_loading;
        let ui = self.ui;

        loading.set(true);

        spawn_local(async move {
            let vars = SignInVars {
                raw_user_details: user_logins,
            };

            let query = r#"
                       mutation SignIn($rawUserDetails: UserLoginsInput!) {
                           signIn(rawUserDetails: $rawUserDetails) {
                                data {
                                    token
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

            let response = perform_mutation_or_query_with_vars::<SignInResponse, SignInVars>(
                None,
                acl_service_api,
                query,
                vars,
            )
            .await;

            match response.get_data() {
                Some(data) => match &data.sign_in {
                    Some(auth_details) => {
                        loading.set(false);
                        if let Some(token) = auth_details.get_data().token {
                            this.token.set(Some(token));
                            // authenticate_with_token manages its own is_loading.
                            this.authenticate_with_token();
                        }
                        return;
                    }
                    None => {}
                },
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    /// Fire-and-forget. Exchanges an OAuth `auth_code` for a token via the
    /// REST `/social-sign-in` endpoint, stores the token, then delegates to
    /// `authenticate_with_token` to verify it — same downstream effect as
    /// the password/GraphQL path.
    pub fn handle_oauth_callback(&self, auth_code: String) {
        let this = *self;
        let loading = self.is_loading;

        loading.set(true);

        spawn_local(async move {
            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let auth_code_body = AuthCode {
                auth_code: Some(auth_code),
            };

            let Ok(response) = reqwest::Client::new()
                .post(&format!("{acl_service_api}/social-sign-in"))
                .json(&auth_code_body)
                .send()
                .await
            else {
                loading.set(false);
                return;
            };

            let Ok(auth_status) = response.json::<RestResponse<AuthDetails>>().await else {
                loading.set(false);
                return;
            };

            loading.set(false);

            if let Some(token) = auth_status.data.and_then(|d| d.token) {
                this.token.set(Some(token));
                // authenticate_with_token manages its own is_loading internally.
                this.authenticate_with_token();
            }
        });
    }

    pub fn headers(&self) -> std::collections::HashMap<String, String> {
        let mut headers = std::collections::HashMap::new();
        headers.insert(
            "Authorization".into(),
            format!("Bearer {}", self.token.get_untracked().unwrap_or_default()),
        );
        headers
    }

    pub fn clear(&self) {
        self.token.set(None);
        self.user_id.set(None);
        self.current_role.set(String::new());
        self.current_role_permissions.set(Vec::new());
    }
}

// ── Context helpers ─────────────────────────────────────────────────────

pub fn provide_auth(ui: UiContext) -> AuthContext {
    let auth_ctx = AuthContext::new(ui);
    provide_context(auth_ctx);
    auth_ctx
}

pub fn use_auth() -> AuthContext {
    expect_context::<AuthContext>()
}
