use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PklServerResponse<T>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    pub header: u64,
    pub response: T,
}

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
