use crate::data_type::{self, DataType};
use crate::errors::{AtentoError, Result};
use serde::{Deserialize, Serialize};

/// Represents an input value for a step, either inline or by reference.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Input {
    /// Reference to a parameter or step output
    Ref {
        #[serde(rename = "ref")]
        ref_: String,
    },
    /// Inline value with explicit type
    Inline {
        #[serde(default, rename = "type")]
        type_: DataType,
        value: serde_yaml::Value,
    },
}

impl Input {
    /// Converts an inline input to a string value.
    ///
    /// # Errors
    /// Returns an error if this is a `Ref` variant or if the value type doesn't match.
    pub fn to_string_value(&self) -> Result<String> {
        match self {
            Self::Inline { value, type_ } => data_type::to_string_value(type_, value),
            Self::Ref { .. } => Err(AtentoError::Execution(
                "Cannot convert Ref directly to string; must resolve first".to_string(),
            )),
        }
    }
}
