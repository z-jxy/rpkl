use crate::value::value::MapImpl;
use std::collections::HashMap;

use crate::internal::msgapi::codes::*;
use serde::Serialize;

use crate::{
    api::{
        evaluator::CREATE_EVALUATOR_REQUEST_ID,
        reader::{PklModuleReader, PklResourceReader},
    },
    internal::msgapi::impl_pkl_message,
    EvaluatorOptions,
};

impl From<&dyn PklModuleReader> for ClientModuleReader {
    fn from(reader: &dyn PklModuleReader) -> Self {
        ClientModuleReader {
            scheme: reader.scheme().to_string(),
            has_hierarchical_uris: reader.has_hierarchical_uris(),
            is_globbable: reader.is_globbable(),
            is_local: reader.is_local(),
        }
    }
}

impl From<&dyn PklResourceReader> for ClientResourceReader {
    fn from(reader: &dyn PklResourceReader) -> Self {
        ClientResourceReader {
            scheme: reader.scheme().to_string(),
            has_hierarchical_uris: reader.has_hierarchical_uris(),
            is_globbable: reader.is_globbable(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateEvaluator<'a> {
    pub request_id: u64,
    pub allowed_modules: Vec<String>,
    pub allowed_resources: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_module_readers: Option<Vec<ClientModuleReader>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_resource_readers: Option<Vec<ClientResourceReader>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<&'a MapImpl<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_resource_readers: Option<&'a HashMap<String, ExternalReader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_module_readers: Option<&'a HashMap<String, ExternalReader>>,
}

impl Default for CreateEvaluator<'_> {
    fn default() -> Self {
        let env_vars: HashMap<String, String> = std::env::vars().collect();
        Self {
            request_id: CREATE_EVALUATOR_REQUEST_ID,
            allowed_modules: vec![
                "pkl:".into(),
                "repl:".into(),
                "file:".into(),
                "https:".into(),
                "package:".into(),
            ],
            allowed_resources: vec![
                "env:".into(),
                "prop:".into(),
                "package:".into(),
                "https:".into(),
                "projectpackage:".into(),
            ],
            env: Some(env_vars),

            client_module_readers: None,
            client_resource_readers: None,
            properties: None,
            external_resource_readers: None,
            external_module_readers: None,
        }
    }
}

impl<'a> From<&'a EvaluatorOptions> for CreateEvaluator<'a> {
    fn from(opts: &'a EvaluatorOptions) -> Self {
        let mut evaluator_message = CreateEvaluator::default();

        /* handle user defined options */
        {
            if let Some(props) = &opts.properties {
                evaluator_message.properties = Some(props);
            }

            if let Some(readers) = opts.client_module_readers.as_ref() {
                for reader in readers.iter() {
                    evaluator_message
                        .allowed_modules
                        .push(reader.scheme().to_string());
                }
                let module_readers: Vec<ClientModuleReader> = readers
                    .iter()
                    .map(|r| {
                        let reader: &dyn PklModuleReader = r.as_ref();
                        reader.into()
                    })
                    .collect();
                evaluator_message.client_module_readers = Some(module_readers);
            }

            if let Some(readers) = opts.client_resource_readers.as_ref() {
                for reader in readers.iter() {
                    evaluator_message
                        .allowed_resources
                        .push(reader.scheme().to_string());
                }
                let resource_readers: Vec<ClientResourceReader> =
                    readers.iter().map(|r| r.as_ref().into()).collect();
                evaluator_message.client_resource_readers = Some(resource_readers);
            }

            if let Some(readers) = &opts.external_resource_readers {
                for uri in readers.keys() {
                    evaluator_message.allowed_resources.push(uri.clone());
                }

                evaluator_message.external_resource_readers = Some(readers);
            }

            if let Some(readers) = &opts.external_module_readers {
                for uri in readers.keys() {
                    evaluator_message.allowed_modules.push(uri.clone());
                }
                evaluator_message.external_module_readers = Some(readers);
            }
        }
        /* */

        evaluator_message
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientModuleReader {
    pub scheme: String,
    pub has_hierarchical_uris: bool,
    pub is_globbable: bool,
    pub is_local: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientResourceReader {
    /// The URI scheme this reader is responsible for reading.
    pub scheme: String,

    /// Tells whether the path part of ths URI has a
    /// [hier-part](https://datatracker.ietf.org/doc/html/rfc3986#section-3).
    ///
    /// An example of a hierarchical URI is `file:///path/to/my/file`, where
    /// `/path/to/my/file` designates a nested path through the `/` character.
    ///
    /// An example of a non-hierarchical URI is `pkl.base`, where the `base` does not denote
    /// any form of hierarchy.
    pub has_hierarchical_uris: bool,

    /// Tells whether this reader supports globbing.
    pub is_globbable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseEvaluator {
    pub evaluator_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateRequest {
    pub request_id: u64,
    pub evaluator_id: i64,
    pub module_uri: String,
}

/// Code: 0x27
///
/// Type: Client Response
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReadResourceResponse {
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
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReadModuleResponse {
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
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InitializeResourceReaderResponse {
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
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InitializeModuleReaderResponse {
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
pub(crate) struct ListResourcesResponse {
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
pub(crate) struct ListModulesResponse {
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

#[derive(Serialize)]
pub struct ExternalReader {
    /// May be specified as an absolute path to an executable
    /// May also be specified as just an executable name, in which case it will be resolved according to the PATH environment variable
    pub executable: String,

    /// Command line arguments that will be passed to the reader process
    pub arguments: Vec<String>,
}

impl_pkl_message!(CreateEvaluator<'a>, CREATE_EVALUATOR);
impl_pkl_message!(EvaluateRequest, EVALUATE_REQUEST);
impl_pkl_message!(CloseEvaluator, CLOSE);

// region: Reader messages
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
// endregion
