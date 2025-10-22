#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::items_after_statements,
    clippy::unnecessary_wraps
)]
mod tests {
    use crate::errors::AtentoError;
    use std::error::Error;

    #[test]
    fn test_io_error_display() {
        let err = AtentoError::Io {
            path: "test.yaml".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
        };
        let display = format!("{err}");
        assert!(display.contains("Failed to read file 'test.yaml'"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_yaml_parse_error_display() {
        let yaml_err = serde_yaml::from_str::<serde_yaml::Value>("invalid: yaml: {").unwrap_err();
        let err = AtentoError::YamlParse {
            context: "workflow.yaml".to_string(),
            source: yaml_err,
        };
        let display = format!("{err}");
        assert!(display.contains("Failed to parse YAML in 'workflow.yaml'"));
    }

    #[test]
    fn test_json_serialize_error_display() {
        let json_err = serde_json::Error::io(std::io::Error::other("simulated IO error"));

        let err = AtentoError::JsonSerialize {
            message: json_err.to_string(),
        };
        let display = format!("{err}");
        assert!(display.contains("Failed to serialize results"));
    }

    #[test]
    fn test_validation_error_display() {
        let err = AtentoError::Validation("Invalid workflow".to_string());
        assert_eq!(
            format!("{err}"),
            "Workflow validation failed: Invalid workflow"
        );
    }

    #[test]
    fn test_execution_error_display() {
        let err = AtentoError::Execution("Step failed".to_string());
        assert_eq!(format!("{err}"), "Workflow execution failed: Step failed");
    }

    #[test]
    fn test_step_execution_error_display() {
        let err = AtentoError::StepExecution {
            step: "build".to_string(),
            reason: "command not found".to_string(),
        };
        assert_eq!(format!("{err}"), "Step 'build' failed: command not found");
    }

    #[test]
    fn test_type_conversion_error_display() {
        let err = AtentoError::TypeConversion {
            expected: "int".to_string(),
            got: "String(\"hello\")".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "Expected int value, got: String(\"hello\")"
        );
    }

    #[test]
    fn test_unresolved_reference_error_display() {
        let err = AtentoError::UnresolvedReference {
            reference: "steps.foo.outputs.bar".to_string(),
            context: "step 'baz'".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "Unresolved reference 'steps.foo.outputs.bar' in step 'baz'"
        );
    }

    #[test]
    fn test_timeout_error_display() {
        let err = AtentoError::Timeout {
            context: "Workflow execution".to_string(),
            timeout_secs: 300,
        };
        assert_eq!(format!("{err}"), "Workflow execution timeout after 300s");
    }

    #[test]
    fn test_runner_error_display() {
        let err = AtentoError::Runner("Failed to create temp file".to_string());
        assert_eq!(format!("{err}"), "Runner error: Failed to create temp file");
    }

    #[test]
    fn test_error_source_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err = AtentoError::Io {
            path: "test.yaml".to_string(),
            source: io_err,
        };
        assert!(err.source().is_some());
    }

    #[test]
    fn test_error_source_yaml() {
        let yaml_err = serde_yaml::from_str::<serde_yaml::Value>("invalid: yaml: {").unwrap_err();
        let err = AtentoError::YamlParse {
            context: "test".to_string(),
            source: yaml_err,
        };
        assert!(err.source().is_some());
    }

    #[test]
    fn test_error_source_json() {
        let json_err = serde_json::Error::io(std::io::Error::other("simulated IO error"));

        let err = AtentoError::JsonSerialize {
            message: json_err.to_string(),
        };
        // JsonSerialize now stores a message string, so source() should be None
        assert!(err.source().is_none());
    }

    #[test]
    fn test_error_source_none_for_others() {
        let err = AtentoError::Validation("test".to_string());
        assert!(err.source().is_none());

        let err = AtentoError::Execution("test".to_string());
        assert!(err.source().is_none());

        let err = AtentoError::StepExecution {
            step: "test".to_string(),
            reason: "test".to_string(),
        };
        assert!(err.source().is_none());
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::Error::io(std::io::Error::other("simulated IO error"));

        let err: AtentoError = json_err.into();
        assert!(matches!(err, AtentoError::JsonSerialize { message: _ }));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> crate::errors::Result<i32> {
            Ok(42)
        }
        assert_eq!(returns_result().unwrap(), 42);

        fn returns_error() -> crate::errors::Result<i32> {
            Err(AtentoError::Validation("test".to_string()))
        }
        assert!(returns_error().is_err());
    }

    #[test]
    fn test_error_debug_format() {
        let err = AtentoError::Validation("test error".to_string());
        let debug = format!("{err:?}");
        assert!(debug.contains("Validation"));
        assert!(debug.contains("test error"));
    }
}
