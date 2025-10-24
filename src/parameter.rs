use crate::data_type::{DataType, to_string_value};
use crate::errors::Result;
use serde::{Deserialize, Serialize};

/// A chain parameter with a typed value.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Parameter {
    #[serde(default, rename = "type")]
    pub type_: DataType,
    pub value: serde_yaml::Value,
}

impl Parameter {
    /// Converts the parameter value to a string according to its `DataType`.
    ///
    /// # Errors
    /// Returns an error if the value type doesn't match the declared `DataType`.
    pub fn to_string_value(&self) -> Result<String> {
        to_string_value(&self.type_, &self.value)
    }
}
