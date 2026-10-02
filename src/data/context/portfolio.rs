use detaxine_ui::utils::forms::{FormDeserializeOptions, deserialize_form_data_with_options};
use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::{File, FormData};

use crate::{
    data::{
        context::{auth::AuthContext, ui::UiContext},
        models::{
            general::{files::UploadedFileResponse, shared::RestResponse},
            graphql::shared::{
                CreatePortfolioItemResponse, CreateProfessionalDetailsResponse,
                CreateResumeItemResponse, CreateUserServiceResponse, CreateUserSkillResponse,
                CreateUserSkillVars, FetchSiteResourcesResponse, ProfessionalDetailsInputVars,
                ResumeItemInputVars, UserPortfolio, UserPortfolioInput, UserPortfolioInputVars,
                UserProfessionalInfo, UserProfessionalInfoInput, UserResume, UserResumeInput,
                UserService, UserServiceInput, UserServiceInputVars, UserSkill, UserSkillInput,
            },
        },
    },
    utils::{
        custom_traits::IntoGlooHeaders,
        errors::{LocalErrorMessage, handle_graphql_errors, unwrap_rest_response},
        graphql_client::{perform_mutation_or_query_with_vars, perform_query_without_vars},
    },
};

const SHARED_SERVICE_API: Option<&str> = option_env!("SHARED_SERVICE_API");
const FILES_SERVICE_API: Option<&str> = option_env!("FILES_SERVICE_API");

// ── Error surfacing ─────────────────────────────────────────────────────

/// Single swap point for user-visible errors. If `LocalErrorMessage` has a
/// `From<String>` impl, replace the body with `ui.set_error(msg.into(), None)`.
#[inline]
fn set_ui_error(ui: &UiContext, msg: impl Into<LocalErrorMessage>) {
    ui.set_error(msg.into(), None);
}

// ── File upload helper ──────────────────────────────────────────────────

/// Uploads `files` to the FILES service, then appends the resulting
/// thumbnail URL to `form_data` under `"thumbnail"`.
///
/// Returns `true` on success. On failure, logs the underlying error and
/// surfaces a user-visible message via `ui.set_error`.
async fn upload_and_set_thumbnail(
    files: &[File],
    form_data: &FormData,
    auth: &AuthContext,
    ui: &UiContext,
) -> bool {
    let Ok(files_form_data) = FormData::new() else {
        leptos::logging::error!("Failed to construct FormData for upload");
        set_ui_error(ui, "Could not prepare the file upload.");
        return false;
    };

    for file in files {
        if let Err(e) = files_form_data.append_with_blob("file", file) {
            leptos::logging::error!("Failed to append Blob: {:?}", e);
            set_ui_error(ui, "Could not attach a file to the upload.");
            return false;
        }
    }

    let Some(files_service_api) = FILES_SERVICE_API else {
        leptos::logging::error!("FILES_SERVICE_API is not configured");
        set_ui_error(ui, "File service is not configured.");
        return false;
    };

    let Ok(request) = gloo_net::http::Request::post(&format!("{files_service_api}/upload/default"))
        .headers(auth.headers().into_gloo_headers())
        .body(files_form_data)
    else {
        leptos::logging::error!("Failed to build upload request");
        set_ui_error(ui, "Could not build the upload request.");
        return false;
    };

    let response = match request.send().await {
        Ok(r) => r,
        Err(err) => {
            leptos::logging::error!("Failed to upload files: {:?}", err);
            set_ui_error(
                ui,
                "Upload failed. Please check your connection and try again.",
            );
            return false;
        }
    };

    let body = match response
        .json::<RestResponse<Vec<UploadedFileResponse>>>()
        .await
    {
        Ok(b) => b,
        Err(err) => {
            leptos::logging::error!("Failed to parse upload response: {:?}", err);
            set_ui_error(ui, "Unexpected response from the file service.");
            return false;
        }
    };

    // `unwrap_rest_response` surfaces the API-level message via `ui` itself.
    let Some(uploaded_files) = unwrap_rest_response(body, ui, auth, None) else {
        return false;
    };

    let Some(first_file) = uploaded_files.first() else {
        leptos::logging::error!("Upload response contained no files");
        set_ui_error(ui, "Upload succeeded but no file was returned.");
        return false;
    };

    if let Err(e) = form_data.append_with_str(
        "thumbnail",
        &format!(
            "{files_service_api}/view/default/{}",
            first_file.original_filename
        ),
    ) {
        leptos::logging::error!("Failed to append thumbnail to form data: {:?}", e);
        set_ui_error(ui, "Could not attach the uploaded file to the form.");
        return false;
    }

    true
}

// ── Context ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Copy)]
pub struct PortfolioContext {
    ui: UiContext,
    auth: AuthContext,
    pub services: RwSignal<Vec<UserService>>,
    pub professions: RwSignal<Vec<UserProfessionalInfo>>,
    pub resume: RwSignal<Vec<UserResume>>,
    pub skills: RwSignal<Vec<UserSkill>>,
    pub portfolio: RwSignal<Vec<UserPortfolio>>,
    pub is_loading: RwSignal<bool>,
    pub professional_details_created_dirty: RwSignal<u64>,
    pub resume_item_created_dirty: RwSignal<u64>,
    pub user_service_created_dirty: RwSignal<u64>,
    pub skill_created_dirty: RwSignal<u64>,
    pub portfolio_item_created_dirty: RwSignal<u64>,
}

impl PortfolioContext {
    pub fn new(ui: UiContext, auth: AuthContext) -> Self {
        Self {
            ui,
            auth,
            services: RwSignal::new(Vec::new()),
            professions: RwSignal::new(Vec::new()),
            resume: RwSignal::new(Vec::new()),
            skills: RwSignal::new(Vec::new()),
            portfolio: RwSignal::new(Vec::new()),
            is_loading: RwSignal::new(false),
            professional_details_created_dirty: RwSignal::new(0),
            resume_item_created_dirty: RwSignal::new(0),
            user_service_created_dirty: RwSignal::new(0),
            skill_created_dirty: RwSignal::new(0),
            portfolio_item_created_dirty: RwSignal::new(0),
        }
    }

    pub fn fetch_services(&self) {
        let services_sig = self.services;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_services_query = r#"
                   query FetchSiteResources {
                        fetchSiteResources {
                            data {
                                services {
                                    title
                                    description
                                    thumbnail
                                    id
                                }
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchSiteResourcesResponse>(
                Some(&headers),
                shared_service_api,
                fetch_services_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_site_resources
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .services
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .to_vec();
                    services_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_professions(&self) {
        let professions_sig = self.professions;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_professions_query = r#"
                   query FetchSiteResources {
                        fetchSiteResources {
                            data {
                                professionalInfo {
                                    description
                                    active
                                    occupation
                                    startDate
                                    id
                                    yearsOfExperience
                                }
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchSiteResourcesResponse>(
                Some(&headers),
                shared_service_api,
                fetch_professions_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_site_resources
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .professional_info
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .to_vec();
                    professions_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_resume(&self) {
        let resume_sig = self.resume;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_resume_query = r#"
                   query FetchSiteResources {
                        fetchSiteResources {
                            data {
                                resume {
                                    title
                                    moreInfo
                                    startDate
                                    endDate
                                    link
                                    section
                                    id
                                    yearsOfExperience
                                    achievements {
                                        id
                                        description
                                    }
                                }
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchSiteResourcesResponse>(
                Some(&headers),
                shared_service_api,
                fetch_resume_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_site_resources
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .resume
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .to_vec();
                    resume_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_skills(&self) {
        let skills_sig = self.skills;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_skills_query = r#"
                   query FetchSiteResources {
                        fetchSiteResources {
                            data {
                                skills {
                                    thumbnail
                                    name
                                    description
                                    level
                                    skillType
                                    startDate
                                    id
                                    yearsOfExperience
                                }
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchSiteResourcesResponse>(
                Some(&headers),
                shared_service_api,
                fetch_skills_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_site_resources
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .skills
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .to_vec();
                    skills_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_portfolio(&self) {
        let portfolio_sig = self.portfolio;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_portfolio_query = r#"
                   query FetchSiteResources {
                        fetchSiteResources {
                            data {
                                portfolio {
                                    title
                                    description
                                    startDate
                                    endDate
                                    link
                                    category
                                    thumbnail
                                    id
                                    yearsOfExperience
                                }
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                   }
               "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchSiteResourcesResponse>(
                Some(&headers),
                shared_service_api,
                fetch_portfolio_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_site_resources
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .portfolio
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .to_vec();
                    portfolio_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    /// Fire-and-forget. Caller observes `professional_details_created_dirty`.
    pub fn create_professional_details(&self, input: UserProfessionalInfoInput) {
        let dirty = self.professional_details_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let input_vars = ProfessionalDetailsInputVars {
                professional_details: input,
            };

            let query = r#"
                       mutation CreateProfessionalDetails($professionalDetails: UserProfessionalInfoInput!) {
                            createProfessionalDetails(professionalDetails: $professionalDetails) {
                                data {
                                    description
                                    active
                                    occupation
                                    startDate
                                    id
                                }
                                metadata {
                                    newAccessToken
                                    requestId
                                }
                            }
                       }
                   "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateProfessionalDetailsResponse,
                ProfessionalDetailsInputVars,
            >(Some(&headers), shared_service_api, query, input_vars)
            .await;

            match response.get_data() {
                Some(_data) => dirty.update(|n| *n += 1),
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    /// Fire-and-forget. Caller observes `resume_item_created_dirty`.
    pub fn create_resume_item(&self, resume_item: UserResumeInput, achievements: Vec<String>) {
        let dirty = self.resume_item_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let input_vars = ResumeItemInputVars {
                resume_item,
                achievements,
            };

            let query = r#"
                       mutation CreateResumeItem($resumeItem: UserResumeInput!, $achievements: [String!]!) {
                            createResumeItem(resumeItem: $resumeItem, achievements: $achievements) {
                                data {
                                    title
                                    moreInfo
                                    startDate
                                    endDate
                                    link
                                    section
                                    id
                                    yearsOfExperience
                                }
                                metadata {
                                    newAccessToken
                                    requestId
                                }
                            }
                       }
                   "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateResumeItemResponse,
                ResumeItemInputVars,
            >(Some(&headers), shared_service_api, query, input_vars)
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

    /// Fire-and-forget. Uploads the file(s) to the FILES service, appends the
    /// resulting thumbnail URL to `form_data`, deserializes it into a
    /// `UserServiceInput`, then calls `createUserService`. Caller observes
    /// `user_service_created_dirty`.
    pub fn create_user_service(&self, files: Vec<File>, form_data: FormData) {
        let dirty = self.user_service_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            if !upload_and_set_thumbnail(&files, &form_data, &auth, &ui).await {
                loading.set(false);
                return;
            }

            let Some(deserialized) = deserialize_form_data_with_options::<UserServiceInput>(
                &form_data,
                &FormDeserializeOptions::default(),
            ) else {
                loading.set(false);
                return;
            };

            let input_vars = UserServiceInputVars {
                user_service: deserialized,
            };

            let query = r#"
                    mutation CreateUserService($userService: UserServiceInput!) {
                        createUserService(userService: $userService) {
                            data {
                                title
                                description
                                thumbnail
                                id
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                    }
                "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateUserServiceResponse,
                UserServiceInputVars,
            >(Some(&headers), shared_service_api, query, input_vars)
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

    /// Fire-and-forget. Uploads the file(s), appends the thumbnail URL to
    /// `form_data`, deserializes into a `UserSkillInput`, then calls
    /// `createSkill`. Caller observes `skill_created_dirty`.
    pub fn create_skill(&self, files: Vec<File>, form_data: FormData) {
        let dirty = self.skill_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            if !upload_and_set_thumbnail(&files, &form_data, &auth, &ui).await {
                loading.set(false);
                return;
            }

            let Some(deserialized) = deserialize_form_data_with_options::<UserSkillInput>(
                &form_data,
                &FormDeserializeOptions::default(),
            ) else {
                loading.set(false);
                return;
            };

            let input_vars = CreateUserSkillVars {
                skill: deserialized,
            };

            let query = r#"
                    mutation CreateSkill($skill: UserSkillInput!) {
                        createSkill(skill: $skill) {
                            data {
                                thumbnail
                                name
                                description
                                level
                                skillType
                                startDate
                                id
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                    }
                "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreateUserSkillResponse,
                CreateUserSkillVars,
            >(Some(&headers), shared_service_api, query, input_vars)
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

    /// Fire-and-forget. Uploads the file(s), appends the thumbnail URL to
    /// `form_data`, deserializes into a `UserPortfolioInput`, then calls
    /// `createPortfolioItem` with the applied skill IDs. Caller observes
    /// `portfolio_item_created_dirty`.
    pub fn create_portfolio_item(
        &self,
        files: Vec<File>,
        form_data: FormData,
        applied_skills: Vec<String>,
    ) {
        let dirty = self.portfolio_item_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            if !upload_and_set_thumbnail(&files, &form_data, &auth, &ui).await {
                loading.set(false);
                return;
            }

            let Some(deserialized) = deserialize_form_data_with_options::<UserPortfolioInput>(
                &form_data,
                &FormDeserializeOptions::default(),
            ) else {
                loading.set(false);
                return;
            };

            let input_vars = UserPortfolioInputVars {
                portfolio_item: deserialized,
                skills: applied_skills,
            };

            let query = r#"
                    mutation CreatePortfolioItem($portfolioItem: UserPortfolioInput!, $skills: [String!]!) {
                        createPortfolioItem(portfolioItem: $portfolioItem, skills: $skills) {
                            data {
                                id
                                title
                                description
                                link
                                startDate
                                category
                                thumbnail
                                skills {
                                    id
                                    thumbnail
                                    name
                                }
                            }
                            metadata {
                                newAccessToken
                                requestId
                            }
                        }
                    }
                "#;

            let Some(shared_service_api) = SHARED_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_mutation_or_query_with_vars::<
                CreatePortfolioItemResponse,
                UserPortfolioInputVars,
            >(Some(&headers), shared_service_api, query, input_vars)
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

pub fn provide_portfolio(ui: UiContext, auth: AuthContext) -> PortfolioContext {
    let portfolio_ctx = PortfolioContext::new(ui, auth);
    provide_context(portfolio_ctx);
    portfolio_ctx
}

pub fn use_portfolio() -> PortfolioContext {
    expect_context::<PortfolioContext>()
}
