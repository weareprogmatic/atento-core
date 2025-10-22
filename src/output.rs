use crate::data_type::DataType;
use serde::{Deserialize, Serialize};

/// Defines how to extract an output value from a step's stdout using a regex pattern.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    /// Regex pattern with at least one capture group
    pub pattern: String,
    #[serde(default, rename = "type")]
    pub type_: DataType,
}
