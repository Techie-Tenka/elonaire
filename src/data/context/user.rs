use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::{
    data::{
        context::{auth::AuthContext, ui::UiContext},
        models::graphql::acl::{
            FetchSingleUserResponse, FetchSingleUserVars, FetchSiteOwnerResponse,
            FetchUsersResponse, SignUpResponse, SignUpVars, User, UserInput,
        },
    },
    utils::{
        errors::handle_graphql_errors,
        graphql_client::{perform_mutation_or_query_with_vars, perform_query_without_vars},
    },
};

const ACL_SERVICE_API: Option<&str> = option_env!("ACL_SERVICE_API");

const USER_BY_ID_QUERY: &str = r#"
    query FetchSingleUser($userId: String!) {
        fetchSingleUser(userId: $userId) {
            data {
                profilePicture
                bio
                id
                fullName
                email
                socials {
                    name
                    url
                }
            }
            metadata {
                requestId
                newAccessToken
            }
        }
    }
"#;

const FETCH_SITE_OWNER_QUERY: &str = r#"
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

#[derive(Clone, Debug, Copy)]
pub struct UserContext {
    ui: UiContext,
    auth: AuthContext,
    pub user_profile: RwSignal<User>,
    /// Populated by `fetch_single_user` for lookups of a user other than the
    /// current session's own profile (e.g. an admin viewing someone else's
    /// record). Kept separate from `user_profile` so that flow can't
    /// accidentally clobber the signed-in user's own data.
    pub viewed_user: RwSignal<Option<User>>,
    pub is_loading: RwSignal<bool>,
    pub user_created_dirty: RwSignal<u64>,
    pub users: RwSignal<Vec<User>>,
}

impl UserContext {
    pub fn new(ui: UiContext, auth: AuthContext) -> Self {
        Self {
            ui,
            auth,
            user_profile: RwSignal::new(User::default()),
            viewed_user: RwSignal::new(None),
            is_loading: RwSignal::new(false),
            user_created_dirty: RwSignal::new(0),
            users: RwSignal::new(Default::default()),
        }
    }

    pub fn set_user_profile(&self, user: User) {
        self.user_profile.set(user);
    }

    pub fn clear(&self) {
        self.user_profile.set(User::default());
        self.viewed_user.set(None);
    }

    /// Fetches the **currently-authenticated user's own profile** from the
    /// ACL service via the `fetchSiteOwnerInfo` endpoint. Identity is
    /// resolved server-side from the `Authorization` header (if `auth` holds
    /// a token) and/or the refresh cookie in the jar — no `userId` is sent.
    ///
    /// This is the session's "who am I" call. It is *not* `fetch_single_user`
    /// — that one looks up an arbitrary user by id. Do not merge them.
    ///
    /// Writes the result into `user_profile`. Callers should typically wait
    /// for `auth.auth_resolved()` to be true before calling this, so that
    /// `bootstrap_session` has had a chance to rotate in a fresh access
    /// token (or confirm no session exists). If called earlier, the request
    /// still goes out — cookies ride along — but relies on the server
    /// accepting cookie auth for this endpoint.
    pub fn fetch_own_profile(&self) {
        let user_profile_sig = self.user_profile;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchSiteOwnerResponse>(
                Some(&headers),
                acl_service_api,
                FETCH_SITE_OWNER_QUERY,
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
                    user_profile_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    /// Fetches an arbitrary user's record (e.g. admin lookup) into
    /// `viewed_user`, leaving `user_profile` untouched. Selection set
    /// varies by call site, so the query is caller-supplied.
    pub fn fetch_single_user(&self, vars: FetchSingleUserVars, query: &'static str) {
        let viewed_user_sig = self.viewed_user;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let Some(acl_service_api) = ACL_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                FetchSingleUserResponse,
                FetchSingleUserVars,
            >(Some(&headers), acl_service_api, query, vars)
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_single_user
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_owned();
                    viewed_user_sig.set(Some(owned_data));
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    /// Async variant for callers that need the value back rather than stashing
    /// it in `viewed_user` (e.g. hydrating comment authors in a loop).
    pub async fn fetch_user_by_id_async(&self, user_id: String) -> Option<User> {
        let acl_service_api = ACL_SERVICE_API?;
        let vars = FetchSingleUserVars { user_id };
        let headers = self.auth.headers();

        let response = perform_mutation_or_query_with_vars::<
            FetchSingleUserResponse,
            FetchSingleUserVars,
        >(Some(&headers), acl_service_api, USER_BY_ID_QUERY, vars)
        .await;

        response.get_data().map(|data| {
            data.fetch_single_user
                .as_ref()
                .unwrap_or(&Default::default())
                .get_data()
                .to_owned()
        })
    }

    pub fn fetch_users(&self) {
        let users_sig = self.users;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_users_query = r#"
                       query FetchUsers {
                            fetchUsers {
                                data {
                                    id
                                    email
                                    status
                                    oauthClient
                                    fullName
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

            let response = perform_query_without_vars::<FetchUsersResponse>(
                Some(&headers),
                acl_service_api,
                fetch_users_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_users
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    users_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    /// Fire-and-forget. Caller observes `user_created_dirty` to react to success.
    pub fn create_user(&self, user: UserInput) {
        let dirty = self.user_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let input_vars = SignUpVars { user };

            let query = r#"
                       mutation SignUp($user: UserInput!) {
                            signUp(user: $user) {
                                data {
                                    id
                                    fullName
                                    email
                                    status
                                    oauthClient
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

            let response = perform_mutation_or_query_with_vars::<SignUpResponse, SignUpVars>(
                Some(&headers),
                acl_service_api,
                query,
                input_vars,
            )
            .await;

            match response.get_data() {
                Some(_data) => {
                    dirty.update(|n| *n += 1);
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

pub fn provide_user(ui: UiContext, auth: AuthContext) -> UserContext {
    let user_ctx = UserContext::new(ui, auth);
    provide_context(user_ctx);
    user_ctx
}

pub fn use_user() -> UserContext {
    expect_context::<UserContext>()
}
