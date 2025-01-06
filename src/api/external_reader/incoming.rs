use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceRequest {
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesRequest {
    /// A number identifying this request.
    request_id: i64,

    /// A number identifying this evaluator.
    evaluator_id: i64,

    /// The URI of the module.
    uri: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListModulesRequest {
    /// A number identifying this request.
    request_id: i64,

    /// A number identifying this evaluator.
    evaluator_id: i64,

    /// The URI of the module.
    uri: String,
}

// impl_pkl_message!(ReadModuleRequest, READ_MODULE_REQUEST);
// impl_pkl_message!(ReadResourceRequest, READ_RESOURCE_REQUEST);
