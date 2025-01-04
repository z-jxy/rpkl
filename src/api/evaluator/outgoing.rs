use std::collections::HashMap;

use codes::*;
use serde::{Deserialize, Serialize};

use super::{ExternalReader, CREATE_EVALUATOR_REQUEST_ID};

pub mod codes {
    pub const CREATE_EVALUATOR: u64 = 0x20;
    pub const CLOSE: u64 = 0x22;
    pub const EVALUATE_REQUEST: u64 = 0x23;

    pub const READ_RESOURCE_REQUEST: u64 = 0x26;
    pub const READ_RESOURCE_RESPONSE: u64 = 0x27;
    pub const READ_MODULE_REQUEST: u64 = 0x28;
    pub const READ_MODULE_RESPONSE: u64 = 0x29;

    pub const INITIALIZE_MODULE_READER_REQUEST: u64 = 0x2e;
    pub const INITIALIZE_MODULE_READER_RESPONSE: u64 = 0x2f;

    pub const INITIALIZE_RESOURCE_READER_REQUEST: u64 = 0x30;
    pub const INITIALIZE_RESOURCE_READER_RESPONSE: u64 = 0x31;
    pub const CLOSE_EXTERNAL_PROCESS: u64 = 0x32;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateEvaluator {
    pub request_id: u64,
    pub allowed_modules: Vec<String>,
    pub allowed_resources: Vec<String>,
    pub client_module_readers: Vec<ClientModuleReader>,
    pub env: Option<HashMap<String, String>>,
    pub properties: Option<HashMap<String, String>>,
    pub external_resource_readers: Option<HashMap<String, ExternalReader>>,
}

impl Default for CreateEvaluator {
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
            client_module_readers: vec![ClientModuleReader {
                scheme: "customfs".to_string(),
                has_hierarchical_uris: true,
                is_globbable: true,
                is_local: true,
            }],
            env: Some(env_vars),
            properties: Some(HashMap::new()),
            external_resource_readers: Some(HashMap::new()),
        }
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
pub enum OutgoingMessage {
    CreateEvaluator(CreateEvaluator),
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

pub fn get_messagepack_header(msg: &OutgoingMessage) -> u64 {
    match msg {
        OutgoingMessage::CreateEvaluator(_) => CREATE_EVALUATOR,
        OutgoingMessage::EvaluateRequest(_) => EVALUATE_REQUEST,
        OutgoingMessage::CloseEvaluator(_) => CLOSE,
    }
}

pub fn pack_msg(msg: OutgoingMessage) -> Vec<u8> {
    let header = get_messagepack_header(&msg);
    let mut serialized_request = Vec::new();
    let _ = &(header, msg)
        .serialize(
            &mut rmp_serde::Serializer::new(&mut serialized_request)
                .with_struct_map()
                .with_binary(),
        )
        .unwrap();
    serialized_request
}
