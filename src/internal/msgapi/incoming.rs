/// Represents a message received from the pkl server/process.
#[derive(Debug, Clone)]
pub struct PklServerMessage {
    pub header: u64,
    pub response: rmpv::Value,
}

// TODO: decide whether to implement deserializing for message or keep the current approach
/*
use serde::{Deserialize, Serialize};

/// Code: 0x102
/// Type: Server Request
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResourceReaderRequest {
    /// A number identifying this request.
    pub request_id: i64,

    /// The scheme of the resource to initialize.
    pub scheme: String,
}

#[derive(Debug, Deserialize)]
pub struct EvaluatorResponse {
    #[serde(rename = "requestId")]
    pub request_id: i64,
    #[serde(rename = "evaluatorId")]
    pub evaluator_id: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadResourceRequest {
    /// A number identifying this request.
    request_id: i64,

    /// A number identifying this evaluator.
    evaluator_id: i64,

    /// The URI of the resource.
    uri: String,
}

/// Code: 0x28
///
/// Type: Server Request
///
/// Read a module at the given URI. This message occurs during the evaluation of an import statement or expression (import/import*), when a scheme matches a client module reader.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadModuleRequest {
    /// A number identifying this request.
    _request_id: i64,

    /// A number identifying this evaluator.
    _evaluator_id: i64,

    /// The URI of the module.
    _uri: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListResourcesRequest {
    /// A number identifying this request.
    _request_id: i64,

    /// A number identifying this evaluator.
    _evaluator_id: i64,

    /// The URI of the module.
    _uri: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListModulesRequest {
    /// A number identifying this request.
    _request_id: i64,

    /// A number identifying this evaluator.
    _evaluator_id: i64,

    /// The URI of the module.
    _uri: String,
}
*/
