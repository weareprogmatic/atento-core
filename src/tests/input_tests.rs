#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::approx_constant)]
mod tests {
    use crate::data_type::DataType;
    use crate::errors::AtentoError;
    use crate::input::Input;
    use serde_yaml::Value;

    #[test]
    fn test_input_ref_to_string_value_fails() {
        let input = Input::Ref {
            ref_: "parameters.foo".to_string(),
        };
        let result = input.to_string_value();
        assert!(result.is_err());
        if let Err(AtentoError::Execution(msg)) = result {
            assert!(msg.contains("Cannot convert Ref"));
            assert!(msg.contains("must resolve first"));
        } else {
            panic!("Expected Execution error");
        }
    }

    #[test]
    fn test_input_inline_string_valid() {
        let input = Input::Inline {
            type_: DataType::String,
            value: Value::String("hello".to_string()),
        };
        let result = input.to_string_value();
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_input_inline_int_valid() {
        let input = Input::Inline {
            type_: DataType::Int,
            value: Value::Number(42.into()),
        };
        let result = input.to_string_value();
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_input_inline_float_valid() {
        let input = Input::Inline {
            type_: DataType::Float,
            value: Value::Number(serde_yaml::Number::from(3.14)),
        };
        let result = input.to_string_value();
        assert_eq!(result.unwrap(), "3.14");
    }

    #[test]
    fn test_input_inline_bool_valid() {
        let input = Input::Inline {
            type_: DataType::Bool,
            value: Value::Bool(true),
        };
        let result = input.to_string_value();
        assert_eq!(result.unwrap(), "true");
    }

    #[test]
    fn test_input_inline_datetime_valid() {
        let input = Input::Inline {
            type_: DataType::DateTime,
            value: Value::String("2024-01-15T10:30:00Z".to_string()),
        };
        let result = input.to_string_value();
        assert_eq!(result.unwrap(), "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_input_inline_type_mismatch() {
        let input = Input::Inline {
            type_: DataType::Int,
            value: Value::String("not a number".to_string()),
        };
        let result = input.to_string_value();
        assert!(result.is_err());
    }

    #[test]
    fn test_input_clone() {
        let input = Input::Ref {
            ref_: "test".to_string(),
        };
        let cloned = input.clone();
        if let (Input::Ref { ref_: r1 }, Input::Ref { ref_: r2 }) = (&input, &cloned) {
            assert_eq!(r1, r2);
        } else {
            panic!("Clone failed");
        }
    }

    #[test]
    fn test_input_debug() {
        let input = Input::Ref {
            ref_: "parameters.foo".to_string(),
        };
        let debug = format!("{input:?}");
        assert!(debug.contains("Ref"));
        assert!(debug.contains("parameters.foo"));
    }

    #[test]
    fn test_input_deserialize_ref() {
        let yaml = r"
ref: parameters.name
";
        let input: Input = serde_yaml::from_str(yaml).unwrap();
        if let Input::Ref { ref_ } = input {
            assert_eq!(ref_, "parameters.name");
        } else {
            panic!("Expected Ref variant");
        }
    }

    #[test]
    fn test_input_deserialize_inline() {
        let yaml = r"
type: string
value: hello
";
        let input: Input = serde_yaml::from_str(yaml).unwrap();
        if let Input::Inline { type_, value } = input {
            assert_eq!(type_, DataType::String);
            assert_eq!(value.as_str().unwrap(), "hello");
        } else {
            panic!("Expected Inline variant");
        }
    }

    #[test]
    fn test_input_deserialize_inline_default_type() {
        let yaml = r"
value: hello
";
        let input: Input = serde_yaml::from_str(yaml).unwrap();
        if let Input::Inline { type_, .. } = input {
            assert_eq!(type_, DataType::String); // Default type
        } else {
            panic!("Expected Inline variant");
        }
    }

    #[test]
    fn test_input_serialize_ref() {
        let input = Input::Ref {
            ref_: "steps.foo.outputs.bar".to_string(),
        };
        let yaml = serde_yaml::to_string(&input).unwrap();
        assert!(yaml.contains("ref"));
        assert!(yaml.contains("steps.foo.outputs.bar"));
    }

    #[test]
    fn test_input_serialize_inline() {
        let input = Input::Inline {
            type_: DataType::Int,
            value: Value::Number(42.into()),
        };
        let yaml = serde_yaml::to_string(&input).unwrap();
        assert!(yaml.contains("type"));
        assert!(yaml.contains("int"));
        assert!(yaml.contains("42"));
    }

    #[test]
    fn test_input_untagged_deserialization() {
        // Test that untagged enum works correctly
        let yaml_ref = r#"{ "ref": "parameters.x" }"#;
        let input_ref: Input = serde_yaml::from_str(yaml_ref).unwrap();
        assert!(matches!(input_ref, Input::Ref { .. }));

        let yaml_inline = r#"{ "type": "int", "value": 42 }"#;
        let input_inline: Input = serde_yaml::from_str(yaml_inline).unwrap();
        assert!(matches!(input_inline, Input::Inline { .. }));
    }

    #[test]
    fn test_input_empty_string() {
        let input = Input::Inline {
            type_: DataType::String,
            value: Value::String(String::new()),
        };
        let result = input.to_string_value();
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_input_zero_values() {
        let input_int = Input::Inline {
            type_: DataType::Int,
            value: Value::Number(0.into()),
        };
        assert_eq!(input_int.to_string_value().unwrap(), "0");

        let input_float = Input::Inline {
            type_: DataType::Float,
            value: Value::Number(serde_yaml::Number::from(0.0)),
        };
        assert_eq!(input_float.to_string_value().unwrap(), "0");
    }
}
