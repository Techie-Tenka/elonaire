use serde::{Deserialize, Serialize};

use crate::utils::errors::LocalRestErrorMessage;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ApiResponse<T> {
    data: T,
    metadata: ApiResponseMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ApiResponseMetadata {
    #[serde(rename = "requestId", alias = "request_id")]
    pub request_id: String,
    #[serde(rename = "newAccessToken", alias = "new_access_token")]
    pub new_access_token: Option<String>,
}

impl<T: Sync + Send + Clone> ApiResponse<T> {
    pub fn get_data(&self) -> T {
        self.data.clone()
    }

    pub fn get_request_id(&self) -> String {
        self.metadata.request_id.clone()
    }

    pub fn get_new_access_token(&self) -> Option<String> {
        self.metadata.new_access_token.as_ref().cloned()
    }
}

// REST
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RestResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<LocalRestErrorMessage>,
    pub metadata: Option<ApiResponseMetadata>,
}
