#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::approx_constant)]
mod tests {
    use crate::data_type::{DataType, to_string_value};
    use crate::errors::AtentoError;
    use serde_yaml::Value;

    #[test]
    fn test_data_type_default() {
        assert_eq!(DataType::default(), DataType::String);
    }

    #[test]
    fn test_data_type_display() {
        assert_eq!(format!("{}", DataType::String), "string");
        assert_eq!(format!("{}", DataType::Int), "int");
        assert_eq!(format!("{}", DataType::Float), "float");
        assert_eq!(format!("{}", DataType::Bool), "bool");
        assert_eq!(format!("{}", DataType::DateTime), "datetime");
    }

    #[test]
    fn test_data_type_equality() {
        assert_eq!(DataType::String, DataType::String);
        assert_ne!(DataType::String, DataType::Int);
    }

    #[test]
    fn test_data_type_clone() {
        let dt = DataType::Float;
        let cloned = dt.clone();
        assert_eq!(dt, cloned);
    }

    #[test]
    fn test_data_type_debug() {
        let dt = DataType::Int;
        let debug = format!("{dt:?}");
        assert!(debug.contains("Int"));
    }

    #[test]
    fn test_to_string_value_string_valid() {
        let value = Value::String("hello".to_string());
        let result = to_string_value(&DataType::String, &value);
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_to_string_value_string_invalid() {
        let value = Value::Number(42.into());
        let result = to_string_value(&DataType::String, &value);
        assert!(result.is_err());
        if let Err(AtentoError::TypeConversion { expected, got }) = result {
            assert_eq!(expected, "string");
            assert!(got.contains("42"));
        } else {
            panic!("Expected TypeConversion error");
        }
    }

    #[test]
    fn test_to_string_value_int_valid() {
        let value = Value::Number(42.into());
        let result = to_string_value(&DataType::Int, &value);
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_to_string_value_int_negative() {
        let value = Value::Number((-42).into());
        let result = to_string_value(&DataType::Int, &value);
        assert_eq!(result.unwrap(), "-42");
    }

    #[test]
    fn test_to_string_value_int_zero() {
        let value = Value::Number(0.into());
        let result = to_string_value(&DataType::Int, &value);
        assert_eq!(result.unwrap(), "0");
    }

    #[test]
    fn test_to_string_value_int_invalid() {
        let value = Value::String("not a number".to_string());
        let result = to_string_value(&DataType::Int, &value);
        assert!(result.is_err());
        if let Err(AtentoError::TypeConversion { expected, .. }) = result {
            assert_eq!(expected, "int");
        }
    }

    #[test]
    fn test_to_string_value_float_valid() {
        let value = Value::Number(serde_yaml::Number::from(3.14));
        let result = to_string_value(&DataType::Float, &value);
        assert_eq!(result.unwrap(), "3.14");
    }

    #[test]
    fn test_to_string_value_float_zero() {
        let value = Value::Number(serde_yaml::Number::from(0.0));
        let result = to_string_value(&DataType::Float, &value);
        assert_eq!(result.unwrap(), "0");
    }

    #[test]
    fn test_to_string_value_float_negative() {
        let value = Value::Number(serde_yaml::Number::from(-2.5));
        let result = to_string_value(&DataType::Float, &value);
        assert_eq!(result.unwrap(), "-2.5");
    }

    #[test]
    fn test_to_string_value_float_invalid() {
        let value = Value::Bool(true);
        let result = to_string_value(&DataType::Float, &value);
        assert!(result.is_err());
        if let Err(AtentoError::TypeConversion { expected, .. }) = result {
            assert_eq!(expected, "float");
        }
    }

    #[test]
    fn test_to_string_value_bool_true() {
        let value = Value::Bool(true);
        let result = to_string_value(&DataType::Bool, &value);
        assert_eq!(result.unwrap(), "true");
    }

    #[test]
    fn test_to_string_value_bool_false() {
        let value = Value::Bool(false);
        let result = to_string_value(&DataType::Bool, &value);
        assert_eq!(result.unwrap(), "false");
    }

    #[test]
    fn test_to_string_value_bool_invalid() {
        let value = Value::String("not a bool".to_string());
        let result = to_string_value(&DataType::Bool, &value);
        assert!(result.is_err());
        if let Err(AtentoError::TypeConversion { expected, .. }) = result {
            assert_eq!(expected, "bool");
        }
    }

    #[test]
    fn test_to_string_value_datetime_valid() {
        let value = Value::String("2024-01-15T10:30:00Z".to_string());
        let result = to_string_value(&DataType::DateTime, &value);
        assert_eq!(result.unwrap(), "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_to_string_value_datetime_invalid() {
        let value = Value::Number(42.into());
        let result = to_string_value(&DataType::DateTime, &value);
        assert!(result.is_err());
        if let Err(AtentoError::TypeConversion { expected, .. }) = result {
            assert_eq!(expected, "datetime string");
        }
    }

    #[test]
    fn test_to_string_value_null_values() {
        let value = Value::Null;

        assert!(to_string_value(&DataType::String, &value).is_err());
        assert!(to_string_value(&DataType::Int, &value).is_err());
        assert!(to_string_value(&DataType::Float, &value).is_err());
        assert!(to_string_value(&DataType::Bool, &value).is_err());
        assert!(to_string_value(&DataType::DateTime, &value).is_err());
    }

    #[test]
    fn test_data_type_serde_roundtrip() {
        let types = vec![
            DataType::String,
            DataType::Int,
            DataType::Float,
            DataType::Bool,
            DataType::DateTime,
        ];

        for dt in types {
            let serialized = serde_json::to_string(&dt).unwrap();
            let deserialized: DataType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(dt, deserialized);
        }
    }

    #[test]
    fn test_data_type_deserialize_lowercase() {
        let json = "\"string\"";
        let dt: DataType = serde_json::from_str(json).unwrap();
        assert_eq!(dt, DataType::String);

        let json = "\"int\"";
        let dt: DataType = serde_json::from_str(json).unwrap();
        assert_eq!(dt, DataType::Int);
    }
}
