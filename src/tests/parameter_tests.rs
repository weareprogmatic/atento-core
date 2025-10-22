#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::data_type::DataType;
    use crate::parameter::Parameter;
    use serde_yaml::Value;

    #[test]
    fn test_parameter_to_string_value_string() {
        let param = Parameter {
            type_: DataType::String,
            value: Value::String("test".to_string()),
        };
        assert_eq!(param.to_string_value().unwrap(), "test");
    }

    #[test]
    fn test_parameter_to_string_value_int() {
        let param = Parameter {
            type_: DataType::Int,
            value: Value::Number(42.into()),
        };
        assert_eq!(param.to_string_value().unwrap(), "42");
    }

    #[test]
    fn test_parameter_to_string_value_float() {
        let param = Parameter {
            type_: DataType::Float,
            value: Value::Number(serde_yaml::Number::from(std::f64::consts::PI)),
        };
        assert_eq!(
            param.to_string_value().unwrap(),
            std::f64::consts::PI.to_string()
        );
    }

    #[test]
    fn test_parameter_to_string_value_bool() {
        let param = Parameter {
            type_: DataType::Bool,
            value: Value::Bool(true),
        };
        assert_eq!(param.to_string_value().unwrap(), "true");
    }

    #[test]
    fn test_parameter_to_string_value_datetime() {
        let param = Parameter {
            type_: DataType::DateTime,
            value: Value::String("2024-01-15T10:30:00Z".to_string()),
        };
        assert_eq!(param.to_string_value().unwrap(), "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_parameter_to_string_value_type_mismatch() {
        let param = Parameter {
            type_: DataType::Int,
            value: Value::String("not a number".to_string()),
        };
        assert!(param.to_string_value().is_err());
    }

    #[test]
    fn test_parameter_clone() {
        let param = Parameter {
            type_: DataType::Int,
            value: Value::Number(42.into()),
        };
        let cloned = param.clone();
        assert_eq!(cloned.type_, param.type_);
    }

    #[test]
    fn test_parameter_debug() {
        let param = Parameter {
            type_: DataType::String,
            value: Value::String("test".to_string()),
        };
        let debug = format!("{param:?}");
        assert!(debug.contains("Parameter"));
    }

    #[test]
    fn test_parameter_deserialize() {
        let yaml = r"
type: int
value: 42
";
        let param: Parameter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(param.type_, DataType::Int);
        assert_eq!(param.value.as_i64().unwrap(), 42);
    }

    #[test]
    fn test_parameter_deserialize_default_type() {
        let yaml = r"
value: hello
";
        let param: Parameter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(param.type_, DataType::String); // Default
    }

    #[test]
    fn test_parameter_serialize() {
        let param = Parameter {
            type_: DataType::Bool,
            value: Value::Bool(false),
        };
        let yaml = serde_yaml::to_string(&param).unwrap();
        assert!(yaml.contains("type"));
        assert!(yaml.contains("bool"));
        assert!(yaml.contains("false"));
    }

    #[test]
    fn test_parameter_roundtrip() {
        let param = Parameter {
            type_: DataType::Float,
            value: Value::Number(serde_yaml::Number::from(std::f64::consts::E)),
        };
        let yaml = serde_yaml::to_string(&param).unwrap();
        let deserialized: Parameter = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(param.type_, deserialized.type_);
    }

    #[test]
    fn test_parameter_empty_string() {
        let param = Parameter {
            type_: DataType::String,
            value: Value::String(String::new()),
        };
        assert_eq!(param.to_string_value().unwrap(), "");
    }

    #[test]
    fn test_parameter_negative_int() {
        let param = Parameter {
            type_: DataType::Int,
            value: Value::Number((-100).into()),
        };
        assert_eq!(param.to_string_value().unwrap(), "-100");
    }

    #[test]
    fn test_parameter_negative_float() {
        let param = Parameter {
            type_: DataType::Float,
            value: Value::Number(serde_yaml::Number::from(-99.99)),
        };
        assert_eq!(param.to_string_value().unwrap(), "-99.99");
    }

    #[test]
    fn test_parameter_zero_int() {
        let param = Parameter {
            type_: DataType::Int,
            value: Value::Number(0.into()),
        };
        assert_eq!(param.to_string_value().unwrap(), "0");
    }

    #[test]
    fn test_parameter_zero_float() {
        let param = Parameter {
            type_: DataType::Float,
            value: Value::Number(serde_yaml::Number::from(0.0)),
        };
        assert_eq!(param.to_string_value().unwrap(), "0");
    }

    #[test]
    fn test_parameter_bool_false() {
        let param = Parameter {
            type_: DataType::Bool,
            value: Value::Bool(false),
        };
        assert_eq!(param.to_string_value().unwrap(), "false");
    }

    #[test]
    fn test_parameter_null_value() {
        let param = Parameter {
            type_: DataType::String,
            value: Value::Null,
        };
        assert!(param.to_string_value().is_err());
    }
}
