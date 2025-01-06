use std::collections::HashMap;

use codes::*;
use serde::{Deserialize, Serialize};

use crate::api::msgapi::macros::impl_pkl_message;

use super::{EvaluatorOptions, ExternalReader, CREATE_EVALUATOR_REQUEST_ID};

pub mod codes {
    pub const CREATE_EVALUATOR: u64 = 0x20;
    pub const CLOSE: u64 = 0x22;
    pub const EVALUATE_REQUEST: u64 = 0x23;

    pub const READ_RESOURCE_REQUEST: u64 = 0x26;
    pub const READ_RESOURCE_RESPONSE: u64 = 0x27;
    pub const READ_MODULE_REQUEST: u64 = 0x28;
    pub const READ_MODULE_RESPONSE: u64 = 0x29;

    pub const LIST_RESOURCES_REQUEST: u64 = 0x2a;
    pub const LIST_RESOURCES_RESPONSE: u64 = 0x2b;
    pub const LIST_MODULES_REQUEST: u64 = 0x2c;
    pub const LIST_MODULES_RESPONSE: u64 = 0x2d;

    pub const INITIALIZE_MODULE_READER_REQUEST: u64 = 0x2e;
    pub const INITIALIZE_MODULE_READER_RESPONSE: u64 = 0x2f;

    pub const INITIALIZE_RESOURCE_READER_REQUEST: u64 = 0x30;
    pub const INITIALIZE_RESOURCE_READER_RESPONSE: u64 = 0x31;
    pub const CLOSE_EXTERNAL_PROCESS: u64 = 0x32;
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
    pub properties: Option<&'a HashMap<String, String>>,

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
                let module_readers: Vec<ClientModuleReader> =
                    readers.iter().map(|r| r.as_ref().into()).collect();
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientModuleReader {
    pub scheme: String,
    pub has_hierarchical_uris: bool,
    pub is_globbable: bool,
    pub is_local: bool,
}

#[derive(Serialize, Deserialize)]
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
#[serde(untagged)]
pub(crate) enum OutgoingMessage<'a> {
    CreateEvaluator(CreateEvaluator<'a>),
    EvaluateRequest(EvaluateRequest),
    CloseEvaluator(CloseEvaluator),
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

/// Code: 0x102
/// Type: Server Request
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResourceReaderRequest {
    /// A number identifying this request.
    pub request_id: i64,

    /// The scheme of the resource to initialize.
    pub scheme: String,
}

impl_pkl_message!(CreateEvaluator<'a>, CREATE_EVALUATOR);
impl_pkl_message!(EvaluateRequest, EVALUATE_REQUEST);
impl_pkl_message!(CloseEvaluator, CLOSE);
