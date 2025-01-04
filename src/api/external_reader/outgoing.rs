use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ReadResourceResponse {
//     /// A number identifying this request.
//     pub request_id: i64,

//     /// A number identifying this evaluator.
//     pub evaluator_id: i64,

//     /// The contents of the resource.
//     pub contents: Vec<u8>,
//     // The description of the error that occured when reading this resource.
//     // pub error: Option<String>, // typealias Binary = Any
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceResponse {
    /// A number identifying this request.
    pub request_id: i64,

    /// A number identifying this evaluator.
    pub evaluator_id: i64,

    /// The contents of the resource.
    pub contents: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceError {
    /// A number identifying this request.
    pub request_id: i64,

    /// A number identifying this evaluator.
    pub evaluator_id: i64,

    /// The description of the error that occured when reading this resource.
    pub error: String,
}
