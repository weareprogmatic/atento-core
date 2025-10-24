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
            context: "chain.yaml".to_string(),
            source: yaml_err,
        };
        let display = format!("{err}");
        assert!(display.contains("Failed to parse YAML in 'chain.yaml'"));
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
        let err = AtentoError::Validation("Invalid chain".to_string());
        assert_eq!(format!("{err}"), "Chain validation failed: Invalid chain");
    }

    #[test]
    fn test_execution_error_display() {
        let err = AtentoError::Execution("Step failed".to_string());
        assert_eq!(format!("{err}"), "Chain execution failed: Step failed");
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
            context: "Chain execution".to_string(),
            timeout_secs: 300,
        };
        assert_eq!(format!("{err}"), "Chain execution timeout after 300s");
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

    #[test]
    fn test_io_error_serialization() {
        // Test the serialize_io_error function (line 48)
        let err = AtentoError::Io {
            path: "test.yaml".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("Io"));
        assert!(json.contains("test.yaml"));
        assert!(json.contains("not found"));
    }

    #[test]
    fn test_yaml_error_serialization() {
        // Test the serialize_yaml_error function (line 58)
        let yaml_err = serde_yaml::from_str::<serde_yaml::Value>("invalid: yaml: {").unwrap_err();
        let err = AtentoError::YamlParse {
            context: "test.yaml".to_string(),
            source: yaml_err,
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("YamlParse"));
        assert!(json.contains("test.yaml"));
    }

    #[test]
    fn test_all_error_variants_serialize() {
        // Test serialization of all variants (covers lines 48, 55, 58, 65)
        let errors = vec![
            AtentoError::Io {
                path: "file.yaml".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
            },
            AtentoError::YamlParse {
                context: "context".to_string(),
                source: serde_yaml::from_str::<serde_yaml::Value>("bad: yaml: {").unwrap_err(),
            },
            AtentoError::JsonSerialize {
                message: "json error".to_string(),
            },
            AtentoError::Validation("validation error".to_string()),
            AtentoError::Execution("execution error".to_string()),
            AtentoError::StepExecution {
                step: "step1".to_string(),
                reason: "failed".to_string(),
            },
            AtentoError::TypeConversion {
                expected: "int".to_string(),
                got: "string".to_string(),
            },
            AtentoError::UnresolvedReference {
                reference: "ref".to_string(),
                context: "ctx".to_string(),
            },
            AtentoError::Timeout {
                context: "timeout".to_string(),
                timeout_secs: 30,
            },
            AtentoError::Runner("runner error".to_string()),
        ];

        for err in errors {
            let json = serde_json::to_string(&err);
            assert!(json.is_ok(), "Failed to serialize error: {err:?}");
        }
    }
}
