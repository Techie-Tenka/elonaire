use crate::{
    data::{
        context::{auth::AuthContext, ui::UiContext},
        models::general::shared::RestResponse,
    },
    utils::graphql_client::{GraphQLResponse, LocalGraphQLErrorMessage},
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

pub fn handle_graphql_errors<T>(
    response: &GraphQLResponse<T>,
    ui: &UiContext,
    redirect_to: Option<&str>,
) {
    let errors = response.get_error();
    errors.iter().for_each(|e| {
        if let Ok(value) = serde_json::to_value(e) {
            if let Ok(err) = serde_json::from_value(value) as Result<LocalGraphQLErrorMessage, _> {
                ui.redirect_to.set(redirect_to.map(|link| link.to_string()));
                ui.error.set(Some(LocalErrorMessage::GraphQL(err)));
            }
        }
    });
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LocalRestErrorBody {
    pub success: bool,
    pub error: LocalRestErrorMessage,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LocalRestErrorMessage {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum LocalErrorMessage {
    GraphQL(LocalGraphQLErrorMessage),
    Rest(LocalRestErrorMessage),
    Client(ClientErrorMessage),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ClientErrorMessage {
    pub message: String,
    pub code: String,
}

impl ClientErrorMessage {
    pub const CODE: &'static str = "CLIENT";

    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: Self::CODE.to_string(),
        }
    }
}

impl LocalErrorMessage {
    pub fn message(&self) -> &str {
        match self {
            LocalErrorMessage::GraphQL(e) => &e.message,
            LocalErrorMessage::Rest(e) => &e.message,
            LocalErrorMessage::Client(e) => &e.message,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            LocalErrorMessage::GraphQL(e) => e
                .extensions
                .as_ref()
                .and_then(|ext| ext.get("code"))
                .map(String::as_str)
                .unwrap_or(ClientErrorMessage::CODE),
            LocalErrorMessage::Rest(e) => &e.code,
            LocalErrorMessage::Client(e) => &e.code,
        }
    }

    pub fn is_client(&self) -> bool {
        self.code() == ClientErrorMessage::CODE
    }
    pub fn is_unauthorized(&self) -> bool {
        self.code() == "401"
    }
    pub fn is_not_found(&self) -> bool {
        self.code() == "404"
    }
    pub fn is_bad_request(&self) -> bool {
        self.code() == "400"
    }
    pub fn is_forbidden(&self) -> bool {
        self.code() == "403"
    }
    pub fn is_unprocessable(&self) -> bool {
        self.code() == "422"
    }
    pub fn is_internal(&self) -> bool {
        self.code() == "500"
    }

    pub fn from_rest_json(json: &str) -> Option<Self> {
        serde_json::from_str::<LocalRestErrorBody>(json)
            .ok()
            .map(|body| LocalErrorMessage::Rest(body.error))
    }
}

impl From<&str> for LocalErrorMessage {
    fn from(s: &str) -> Self {
        LocalErrorMessage::Client(ClientErrorMessage::new(s))
    }
}

impl From<String> for LocalErrorMessage {
    fn from(s: String) -> Self {
        LocalErrorMessage::Client(ClientErrorMessage::new(s))
    }
}

pub fn unwrap_rest_response<T>(
    body: RestResponse<T>,
    ui: &UiContext,
    auth: &AuthContext,
    redirect_to: Option<&str>,
) -> Option<T> {
    if !body.success {
        let error = body.error.map(LocalErrorMessage::Rest).unwrap_or_else(|| {
            LocalErrorMessage::Rest(LocalRestErrorMessage {
                code: "UNKNOWN".into(),
                message: "An unknown error occurred".into(),
            })
        });
        ui.redirect_to.set(redirect_to.map(|link| link.to_string()));
        ui.error.set(Some(error));
        return None;
    }
    if let Some(metadata) = body.metadata {
        if let Some(token) = metadata.new_access_token {
            auth.token.set(Some(token));
        }
    }
    body.data
}
