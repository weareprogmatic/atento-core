#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::data_type::DataType;
    use crate::output::Output;

    #[test]
    fn test_output_creation() {
        let output = Output {
            pattern: r"result: (\d+)".to_string(),
            type_: DataType::Int,
        };
        assert_eq!(output.pattern, r"result: (\d+)");
        assert_eq!(output.type_, DataType::Int);
    }

    #[test]
    fn test_output_clone() {
        let output = Output {
            pattern: r"value: (.+)".to_string(),
            type_: DataType::String,
        };
        let cloned = output.clone();
        assert_eq!(output.pattern, cloned.pattern);
        assert_eq!(output.type_, cloned.type_);
    }

    #[test]
    fn test_output_debug() {
        let output = Output {
            pattern: r"(\w+)".to_string(),
            type_: DataType::Bool,
        };
        let debug = format!("{output:?}");
        assert!(debug.contains("Output"));
        assert!(debug.contains(r"(\\w+)"));
    }

    #[test]
    fn test_output_deserialize() {
        let yaml = r#"
pattern: "result: (\\d+)"
type: int
"#;
        let output: Output = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(output.pattern, r"result: (\d+)");
        assert_eq!(output.type_, DataType::Int);
    }

    #[test]
    fn test_output_deserialize_default_type() {
        let yaml = r#"
pattern: "value: (.+)"
"#;
        let output: Output = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(output.pattern, "value: (.+)");
        assert_eq!(output.type_, DataType::String); // Default
    }

    #[test]
    fn test_output_serialize() {
        let output = Output {
            pattern: r"(\d+\.\d+)".to_string(),
            type_: DataType::Float,
        };
        let yaml = serde_yaml::to_string(&output).unwrap();
        assert!(yaml.contains("pattern"));
        assert!(yaml.contains("type"));
        assert!(yaml.contains("float"));
    }

    #[test]
    fn test_output_roundtrip() {
        let output = Output {
            pattern: r"timestamp: (.+)".to_string(),
            type_: DataType::DateTime,
        };
        let yaml = serde_yaml::to_string(&output).unwrap();
        let deserialized: Output = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(output.pattern, deserialized.pattern);
        assert_eq!(output.type_, deserialized.type_);
    }

    #[test]
    fn test_output_empty_pattern() {
        let output = Output {
            pattern: String::new(),
            type_: DataType::String,
        };
        assert_eq!(output.pattern, "");
    }

    #[test]
    fn test_output_complex_regex_pattern() {
        let output = Output {
            pattern: r"^ERROR:\s+(.+?)$".to_string(),
            type_: DataType::String,
        };
        assert!(output.pattern.contains("ERROR"));
    }

    #[test]
    fn test_output_all_data_types() {
        let types = vec![
            DataType::String,
            DataType::Int,
            DataType::Float,
            DataType::Bool,
            DataType::DateTime,
        ];

        for dt in types {
            let output = Output {
                pattern: r"(.+)".to_string(),
                type_: dt.clone(),
            };
            assert_eq!(output.type_, dt);
        }
    }

    #[test]
    fn test_output_whitespace_in_pattern() {
        let output = Output {
            pattern: r"value:\s+(\d+)".to_string(),
            type_: DataType::Int,
        };
        assert!(output.pattern.contains(r"\s+"));
    }
}
