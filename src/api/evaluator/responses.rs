use serde::{Deserialize, Serialize};

use super::outgoing::ClientResourceReader;

// TODO: move to a separate external_reader mod
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
    pub spec: Option<ClientResourceReader>,
}

// #[derive(Debug, Clone)]
// pub struct PklServerResponse<T>
// where
//     T: serde::de::DeserializeOwned + std::fmt::Debug,
// {
//     pub header: u64,
//     pub response: T,
// }

#[derive(Debug, Clone)]
pub struct PklServerResponse2<T> {
    pub header: u64,
    pub response: T,
}

#[derive(Debug, Clone)]
pub struct PklServerResponseRaw {
    pub header: u64,
    pub response: rmpv::Value,
}

#[derive(Debug, Deserialize)]
pub struct EvaluatorResponse {
    #[serde(rename = "requestId")]
    pub request_id: i64,
    #[serde(rename = "evaluatorId")]
    pub evaluator_id: i64, // Adjusted to i32
}
