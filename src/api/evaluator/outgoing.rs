use std::collections::HashMap;

use codes::*;
use serde::{Deserialize, Serialize};

use super::{ExternalReader, CREATE_EVALUATOR_REQUEST_ID};

mod codes {
    pub const CREATE_EVALUATOR: u64 = 0x20;
    pub const EVALUATE_REQUEST: u64 = 0x23;
    pub const CLOSE: u64 = 0x22;

    pub const INITIALIZE_RESOURCE_READER_REQUEST: u64 = 0x102;
    pub const CLOSE_EXTERNAL_PROCESS: u64 = 0x104;
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
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResourceReaderRequest {
    /// A number identifying this request.
    request_id: i64,

    /// The scheme of the resource to initialize.
    scheme: String,
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
