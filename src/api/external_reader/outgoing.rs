use serde::{Deserialize, Serialize};

use crate::api::{
    evaluator::outgoing::{
        codes::{
            INITIALIZE_MODULE_READER_RESPONSE, INITIALIZE_RESOURCE_READER_RESPONSE,
            LIST_MODULES_RESPONSE, LIST_RESOURCES_RESPONSE, READ_MODULE_RESPONSE,
            READ_RESOURCE_RESPONSE,
        },
        ClientModuleReader, ClientResourceReader,
    },
    msgapi::macros::impl_pkl_message,
};

/// Code: 0x27
///
/// Type: Client Response
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceResponse {
    /// A number identifying this request.
    pub request_id: i64,

    /// A number identifying this evaluator.
    pub evaluator_id: i64,

    /// The contents of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<Vec<u8>>,

    /// The description of the error that occured when reading this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Code: 0x29
///
/// Type: Client Response
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadModuleResponse {
    /// A number identifying this request.
    pub request_id: i64,

    /// A number identifying this evaluator.
    pub evaluator_id: i64,

    /// The contents of the module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<String>,

    /// The description of the error that occured when reading this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Code: 0x103
/// Type: Client Response
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResourceReaderResponse {
    /// A number identifying this request.
    pub request_id: i64,

    /// Client-side resource reader spec.
    ///
    /// Null when the external process does not implement the requested scheme.
    /// [ClientResourceReader] is defined at https://pkl-lang.org/main/current/bindings-specification/message-passing-api.html#create-evaluator-request
    ///
    ///
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<ClientResourceReader>,
}

/// Code: 0x2f
/// Type: Client Response
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeModuleReaderResponse {
    /// A number identifying this request.
    pub request_id: i64,

    /// Client-side resource reader spec.
    ///
    /// Null when the external process does not implement the requested scheme.
    /// [ClientResourceReader] is defined at https://pkl-lang.org/main/current/bindings-specification/message-passing-api.html#create-evaluator-request
    ///
    ///
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<ClientModuleReader>,
}

/// Code: 0x2a
/// Type: Server Request
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesResponse {
    /// A number identifying this request.
    pub request_id: i64,

    /// A number identifying this evaluator.
    pub evaluator_id: i64,

    /// The elements at the provided base path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_elements: Option<Vec<PathElements>>,

    /// The description of the error that occured when listing elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Code: 0x2c
/// Type: Server Request
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListModulesResponse {
    /// A number identifying this request.
    pub request_id: i64,

    /// A number identifying this evaluator.
    pub evaluator_id: i64,

    /// The elements at the provided base path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_elements: Option<Vec<PathElements>>,

    /// The description of the error that occured when listing elements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathElements {
    /// The name of the element at this path
    pub name: String,

    /// Tells whether the element is a directory.
    pub is_directory: bool,
}

impl_pkl_message!(
    InitializeResourceReaderResponse,
    INITIALIZE_RESOURCE_READER_RESPONSE
);

impl_pkl_message!(
    InitializeModuleReaderResponse,
    INITIALIZE_MODULE_READER_RESPONSE
);

impl_pkl_message!(ReadResourceResponse, READ_RESOURCE_RESPONSE);
impl_pkl_message!(ReadModuleResponse, READ_MODULE_RESPONSE);
impl_pkl_message!(ListResourcesResponse, LIST_RESOURCES_RESPONSE);
impl_pkl_message!(ListModulesResponse, LIST_MODULES_RESPONSE);
