pub struct ReadResource {
    /// A number identifying this request.
    requestId: i64,

    /// A number identifying this evaluator.
    evaluatorId: i64,

    /// The URI of the resource.
    uri: String,
}
