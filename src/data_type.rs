use crate::errors::{AtentoError, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::fmt;

/// Represents the data type of a parameter, input, or output value.
#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    /// UTF-8 string value
    String,
    /// 64-bit signed integer
    Int,
    /// 64-bit floating point number
    Float,
    /// Boolean value (true/false)
    Bool,
    /// ISO 8601 datetime string
    DateTime,
}

impl Default for DataType {
    fn default() -> Self {
        Self::String
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::String => "string",
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::DateTime => "datetime",
        };
        write!(f, "{s}")
    }
}

/// Converts a YAML value to a string representation according to the specified data type.
///
/// # Errors
/// Returns an error if the value type doesn't match the expected `DataType`.
pub fn to_string_value(type_: &DataType, value: &Value) -> Result<String> {
    match type_ {
        DataType::String => {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| AtentoError::TypeConversion {
                    expected: "string".to_string(),
                    got: format!("{value:?}"),
                })
        }

        DataType::Int => {
            value
                .as_i64()
                .map(|i| i.to_string())
                .ok_or_else(|| AtentoError::TypeConversion {
                    expected: "int".to_string(),
                    got: format!("{value:?}"),
                })
        }

        DataType::Float => {
            value
                .as_f64()
                .map(|f| f.to_string())
                .ok_or_else(|| AtentoError::TypeConversion {
                    expected: "float".to_string(),
                    got: format!("{value:?}"),
                })
        }

        DataType::Bool => {
            value
                .as_bool()
                .map(|b| b.to_string())
                .ok_or_else(|| AtentoError::TypeConversion {
                    expected: "bool".to_string(),
                    got: format!("{value:?}"),
                })
        }

        DataType::DateTime => {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| AtentoError::TypeConversion {
                    expected: "datetime string".to_string(),
                    got: format!("{value:?}"),
                })
        }
    }
}
