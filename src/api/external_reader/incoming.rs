use serde::{Deserialize, Serialize};

use crate::api::{
    evaluator::outgoing::codes::READ_MODULE_REQUEST, msgapi::macros::impl_pkl_message,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResource {
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
pub struct ReadModuleRequest {
    /// A number identifying this request.
    request_id: i64,

    /// A number identifying this evaluator.
    evaluator_id: i64,

    /// The URI of the module.
    uri: String,
}

impl_pkl_message!(ReadModuleRequest, READ_MODULE_REQUEST);
