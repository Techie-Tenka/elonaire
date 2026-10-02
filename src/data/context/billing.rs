use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::{File, FormData};

use crate::{
    data::{
        context::{auth::AuthContext, ui::UiContext},
        models::{
            general::{files::UploadedFileResponse, shared::RestResponse},
            graphql::shared::{
                CreateRatecardResponse, CreateRatecardVars, CreateServiceRateResponse,
                CreateServiceRateVars, CreateServiceRequestResponse, CreateServiceRequestVars,
                Currency, FetchBillingRateResponse, FetchBillingRateVars, FetchCurrenciesResponse,
                FetchRatecardsResponse, FetchServiceRatesResponse, FetchServiceRequestsResponse,
                Ratecard, RatecardInput, RatecardInputMetadata, ServiceRate, ServiceRateInput,
                ServiceRateInputMetadata, ServiceRequest, ServiceRequestInput,
                ServiceRequestInputMetadata,
            },
        },
    },
    utils::{
        custom_traits::IntoGlooHeaders,
        errors::{handle_graphql_errors, unwrap_rest_response},
        graphql_client::{perform_mutation_or_query_with_vars, perform_query_without_vars},
    },
};

const SHARED_SERVICE_API: Option<&str> = option_env!("SHARED_SERVICE_API");
const PAYMENTS_SERVICE_API: Option<&str> = option_env!("PAYMENTS_SERVICE_API");
const FILES_SERVICE_API: Option<&str> = option_env!("FILES_SERVICE_API");

// ── File upload helper ──────────────────────────────────────────────────

/// Uploads supporting documents to the FILES service and returns the
/// uploaded-file records. Returns `None` on failure, having logged the
/// underlying error and surfaced a user-visible message via `ui`.
async fn upload_supporting_docs(
    files: &[File],
    auth: &AuthContext,
    ui: &UiContext,
    redirect_to: Option<&str>,
) -> Option<Vec<UploadedFileResponse>> {
    let Ok(files_form_data) = FormData::new() else {
        leptos::logging::error!("Failed to construct FormData for upload");
        ui.set_client_error("Could not prepare the file upload.".to_string());
        return None;
    };

    for file in files {
        if let Err(e) = files_form_data.append_with_blob("file", file) {
            leptos::logging::error!("Failed to append Blob: {:?}", e);
            ui.set_client_error("Could not attach a file to the upload.".to_string());
            return None;
        }
    }

    let Some(files_service_api) = FILES_SERVICE_API else {
        leptos::logging::error!("FILES_SERVICE_API is not configured");
        ui.set_client_error("File service is not configured.".to_string());
        return None;
    };

    let Ok(request) = gloo_net::http::Request::post(&format!("{files_service_api}/upload/default"))
        .headers(auth.headers().into_gloo_headers())
        .body(files_form_data)
    else {
        leptos::logging::error!("Failed to build upload request");
        ui.set_client_error("Could not build the upload request.".to_string());
        return None;
    };

    let response = match request.send().await {
        Ok(r) => r,
        Err(err) => {
            leptos::logging::error!("Failed to upload files: {:?}", err);
            ui.set_client_error(
                "Upload failed. Please check your connection and try again.".to_string(),
            );
            return None;
        }
    };

    let body = match response
        .json::<RestResponse<Vec<UploadedFileResponse>>>()
        .await
    {
        Ok(b) => b,
        Err(err) => {
            leptos::logging::error!("Failed to parse upload response: {:?}", err);
            ui.set_client_error("Unexpected response from the file service.".to_string());
            return None;
        }
    };

    // `unwrap_rest_response` surfaces the API-level message via `ui` itself.
    unwrap_rest_response(body, ui, auth, redirect_to)
}

// ── Context ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Copy)]
pub struct BillingContext {
    ui: UiContext,
    auth: AuthContext,
    pub ratecards: RwSignal<Vec<Ratecard>>,
    pub service_rates: RwSignal<Vec<ServiceRate>>,
    pub currencies: RwSignal<Vec<Currency>>,
    pub service_requests: RwSignal<Vec<ServiceRequest>>,
    pub is_loading: RwSignal<bool>,
    pub ratecard_created_dirty: RwSignal<u64>,
    pub service_request_created_dirty: RwSignal<u64>,
    pub service_rate_created_dirty: RwSignal<u64>,
}

impl BillingContext {
    pub fn new(ui: UiContext, auth: AuthContext) -> Self {
        Self {
            ui,
            auth,
            ratecards: RwSignal::new(Vec::new()),
            service_rates: RwSignal::new(Vec::new()),
            currencies: RwSignal::new(Vec::new()),
            service_requests: RwSignal::new(Vec::new()),
            is_loading: RwSignal::new(false),
            ratecard_created_dirty: RwSignal::new(0),
            service_request_created_dirty: RwSignal::new(0),
            service_rate_created_dirty: RwSignal::new(0),
        }
    }

    pub fn fetch_ratecards(&self) {
        let ratecards_sig = self.ratecards;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_ratecards_query = r#"
                query FetchRatecards {
                    fetchRatecards {
                        data {
                            name
                            createdAt
                            updatedAt
                            id
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

            let response = perform_query_without_vars::<FetchRatecardsResponse>(
                Some(&headers),
                shared_service_api,
                fetch_ratecards_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_ratecards
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    ratecards_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_billing_rate(
        &self,
        vars: FetchBillingRateVars,
        on_done: Callback<Option<String>>,
    ) {
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_billing_rate_query = r#"
                query FetchBillingRate($billingInterval: BillingInterval!, $serviceIds: [String!]!) {
                    fetchBillingRate(billingInterval: $billingInterval, serviceIds: $serviceIds) {
                        data
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
                FetchBillingRateResponse,
                FetchBillingRateVars,
            >(
                Some(&headers),
                shared_service_api,
                fetch_billing_rate_query,
                vars,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_billing_rate
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_owned();
                    on_done.run(Some(owned_data));
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                    on_done.run(None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_service_rates(&self) {
        let service_rates_sig = self.service_rates;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let fetch_service_rates_query = r#"
                query FetchServiceRates {
                    fetchServiceRates {
                        data {
                            hourWeek
                            createdAt
                            updatedAt
                            id
                            baseRate
                            service {
                                title
                                description
                                thumbnail
                                id
                            }
                            currencyId {
                                currencyId
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

            let response = perform_query_without_vars::<FetchServiceRatesResponse>(
                Some(&headers),
                shared_service_api,
                fetch_service_rates_query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_service_rates
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    service_rates_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_currencies(&self) {
        let currencies_sig = self.currencies;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let query = r#"
                query FetchCurrencies {
                    fetchCurrencies {
                        data {
                            code
                            numeric
                            name
                            symbol
                            createdAt
                            updatedAt
                            id
                        }
                        metadata {
                            newAccessToken
                            requestId
                        }
                    }
                }
               "#;

            let Some(payments_service_api) = PAYMENTS_SERVICE_API else {
                loading.set(false);
                return;
            };

            let headers = auth.headers();

            let response = perform_query_without_vars::<FetchCurrenciesResponse>(
                Some(&headers),
                payments_service_api,
                query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_currencies
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    currencies_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    pub fn fetch_service_requests(&self) {
        let service_requests_sig = self.service_requests;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let query = r#"
                query FetchServiceRequests {
                    fetchServiceRequests {
                        data {
                            description
                            startDate
                            endDate
                            createdAt
                            updatedAt
                            id
                            supportingDocs {
                                fileId
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

            let response = perform_query_without_vars::<FetchServiceRequestsResponse>(
                Some(&headers),
                shared_service_api,
                query,
            )
            .await;

            match response.get_data() {
                Some(data) => {
                    let owned_data = data
                        .fetch_service_requests
                        .as_ref()
                        .unwrap_or(&Default::default())
                        .get_data()
                        .to_vec();
                    service_requests_sig.set(owned_data);
                }
                None => {
                    handle_graphql_errors(&response, &ui, None);
                }
            }

            loading.set(false);
        });
    }

    /// Fire-and-forget. Caller observes `ratecard_created_dirty` to react to success.
    pub fn create_ratecard(&self, ratecard_input: RatecardInput, service_ids: Vec<String>) {
        let dirty = self.ratecard_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let input_vars = CreateRatecardVars {
                ratecard_input,
                ratecard_input_metadata: RatecardInputMetadata { service_ids },
            };

            let query = r#"
                       mutation CreateRatecard($ratecardInput: RatecardInput!, $ratecardInputMetadata: RatecardInputMetadata!) {
                            createRatecard(ratecardInput: $ratecardInput, ratecardInputMetadata: $ratecardInputMetadata) {
                                data {
                                    name
                                    createdAt
                                    updatedAt
                                    id
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

            let response = perform_mutation_or_query_with_vars::<
                CreateRatecardResponse,
                CreateRatecardVars,
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

    /// Fire-and-forget. Uploads the supporting documents to the FILES service,
    /// builds the `ServiceRequestInputMetadata` from the uploaded file IDs +
    /// the applied `service_ids`, then calls `createServiceRequest`. Caller
    /// observes `service_request_created_dirty` to react to success.
    pub fn create_service_request(
        &self,
        files: Vec<File>,
        service_request_input: ServiceRequestInput,
        service_ids: Vec<String>,
        redirect_to: Option<String>,
    ) {
        let dirty = self.service_request_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let Some(uploaded_files) =
                upload_supporting_docs(&files, &auth, &ui, redirect_to.as_deref()).await
            else {
                loading.set(false);
                return;
            };

            let metadata = ServiceRequestInputMetadata {
                supporting_docs_file_ids: uploaded_files
                    .iter()
                    .map(|f| f.file_id.clone())
                    .collect(),
                service_ids,
            };

            let input_vars = CreateServiceRequestVars {
                service_request_input,
                service_request_input_metadata: metadata,
            };

            let query = r#"
                    mutation CreateServiceRequest(
                        $serviceRequestInput: ServiceRequestInput!,
                        $serviceRequestInputMetadata: ServiceRequestInputMetadata!
                    ) {
                        createServiceRequest(
                            serviceRequestInput: $serviceRequestInput,
                            serviceRequestInputMetadata: $serviceRequestInputMetadata
                        ) {
                            data {
                                description
                                startDate
                                engagementLength
                                createdAt
                                updatedAt
                                id
                                supportingDocs {
                                    id
                                    fileId
                                }
                            }
                            metadata {
                                requestId
                                newAccessToken
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
                CreateServiceRequestResponse,
                CreateServiceRequestVars,
            >(Some(&headers), shared_service_api, query, input_vars)
            .await;

            match response.get_data() {
                Some(_data) => {
                    dirty.update(|n| *n += 1);
                }
                None => {
                    handle_graphql_errors(&response, &ui, redirect_to.as_deref());
                }
            }

            loading.set(false);
        });
    }

    /// Fire-and-forget. Caller observes `service_rate_created_dirty`.
    pub fn create_service_rate(
        &self,
        service_rate_input: ServiceRateInput,
        service_id: String,
        currency_id: String,
    ) {
        let dirty = self.service_rate_created_dirty;
        let loading = self.is_loading;
        let ui = self.ui;
        let auth = self.auth;

        loading.set(true);

        spawn_local(async move {
            let input_vars = CreateServiceRateVars {
                service_rate_input,
                service_rate_input_metadata: ServiceRateInputMetadata {
                    service_id,
                    currency_id,
                },
            };

            let query = r#"
                       mutation CreateServiceRate($serviceRateInput: ServiceRateInput!, $serviceRateInputMetadata: ServiceRateInputMetadata!) {
                            createServiceRate(serviceRateInput: $serviceRateInput, serviceRateInputMetadata: $serviceRateInputMetadata) {
                               data {
                                    hourWeek
                                    createdAt
                                    updatedAt
                                    id
                                    baseRate
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
                CreateServiceRateResponse,
                CreateServiceRateVars,
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

pub fn provide_billing(ui: UiContext, auth: AuthContext) -> BillingContext {
    let billing_ctx = BillingContext::new(ui, auth);
    provide_context(billing_ctx);
    billing_ctx
}

pub fn use_billing() -> BillingContext {
    expect_context::<BillingContext>()
}
