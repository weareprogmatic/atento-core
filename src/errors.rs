use serde::Serialize;
use std::fmt;

/// The main error type for the Atento chain engine.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum AtentoError {
    /// I/O error when reading files
    Io {
        path: String,
        #[serde(serialize_with = "serialize_io_error")]
        source: std::io::Error,
    },

    /// YAML parsing error
    YamlParse {
        context: String,
        #[serde(serialize_with = "serialize_yaml_error")]
        source: serde_yaml::Error,
    },

    /// JSON serialization error
    JsonSerialize { message: String },

    /// Chain validation error
    Validation(String),

    /// Chain execution error
    Execution(String),

    /// Step execution error
    StepExecution { step: String, reason: String },

    /// Data type conversion error
    TypeConversion { expected: String, got: String },

    /// Reference resolution error
    UnresolvedReference { reference: String, context: String },

    /// Timeout error
    Timeout { context: String, timeout_secs: u64 },

    /// Script runner error
    Runner(String),
}

// Custom serializers for non-serializable error types
fn serialize_io_error<S>(
    error: &std::io::Error,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&error.to_string())
}

fn serialize_yaml_error<S>(
    error: &serde_yaml::Error,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&error.to_string())
}

// Note: JsonSerialize variant stores a message string, see From impl below.

impl fmt::Display for AtentoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "Failed to read file '{path}': {source}")
            }
            Self::YamlParse { context, source } => {
                write!(f, "Failed to parse YAML in '{context}': {source}")
            }
            Self::JsonSerialize { message } => {
                write!(f, "Failed to serialize results: {message}")
            }
            Self::Validation(msg) => {
                write!(f, "Chain validation failed: {msg}")
            }
            Self::Execution(msg) => {
                write!(f, "Chain execution failed: {msg}")
            }
            Self::StepExecution { step, reason } => {
                write!(f, "Step '{step}' failed: {reason}")
            }
            Self::TypeConversion { expected, got } => {
                write!(f, "Expected {expected} value, got: {got}")
            }
            Self::UnresolvedReference { reference, context } => {
                write!(f, "Unresolved reference '{reference}' in {context}")
            }
            Self::Timeout {
                context,
                timeout_secs,
            } => {
                write!(f, "{context} timeout after {timeout_secs}s")
            }
            Self::Runner(msg) => {
                write!(f, "Runner error: {msg}")
            }
        }
    }
}

impl std::error::Error for AtentoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::YamlParse { source, .. } => Some(source),
            // JsonSerialize now contains a message string; no underlying error to return as source
            _ => None,
        }
    }
}

impl From<serde_json::Error> for AtentoError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonSerialize {
            message: err.to_string(),
        }
    }
}

/// Type alias for Results using `AtentoError`
pub type Result<T> = std::result::Result<T, AtentoError>;
