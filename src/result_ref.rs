use serde::Deserialize;

/// A reference to a step output that should be included in the chain results.
#[derive(Debug, Clone, Deserialize)]
pub struct ResultRef {
    #[serde(rename = "ref")]
    pub ref_: String,
}
